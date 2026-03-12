//! Lightweight time-series metrics backed by SQLite.
//!
//! Stores per-resource I/O rates (bytes/s) at 5-second intervals.
//! The middleware writes samples in a background task; the frontend
//! fetches history on page load so charts are immediately populated.

use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;
use tracing::{info, warn};

const DB_PATH: &str = "/var/lib/nasty/metrics.db";
const RETENTION_SECS: i64 = 86400; // 24 hours

#[derive(Debug, Serialize)]
pub struct IoSample {
    /// Unix epoch milliseconds
    pub ts: i64,
    pub in_rate: f64,
    pub out_rate: f64,
}

pub struct MetricsDb {
    conn: Mutex<Connection>,
}

impl MetricsDb {
    pub fn open() -> Result<Self, String> {
        let conn = Connection::open(DB_PATH)
            .map_err(|e| format!("failed to open metrics db: {e}"))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS io_samples (
                 ts      INTEGER NOT NULL,
                 kind    TEXT NOT NULL,
                 name    TEXT NOT NULL,
                 in_rate REAL NOT NULL,
                 out_rate REAL NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_io_lookup
                 ON io_samples(kind, name, ts);",
        )
        .map_err(|e| format!("failed to initialize metrics schema: {e}"))?;

        info!("Metrics database opened at {DB_PATH}");
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Insert a batch of I/O rate samples.
    pub fn insert(&self, kind: &str, samples: &[(&str, f64, f64)]) {
        let conn = self.conn.lock().unwrap();
        let ts = now_ms();
        let mut stmt = match conn.prepare(
            "INSERT INTO io_samples (ts, kind, name, in_rate, out_rate) VALUES (?1, ?2, ?3, ?4, ?5)",
        ) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to prepare metrics insert: {e}");
                return;
            }
        };
        for &(name, in_rate, out_rate) in samples {
            if let Err(e) = stmt.execute(rusqlite::params![ts, kind, name, in_rate, out_rate]) {
                warn!("Failed to insert metric for {kind}/{name}: {e}");
            }
        }
    }

    /// Prune samples older than the retention period.
    pub fn prune(&self) {
        let cutoff = now_ms() - RETENTION_SECS * 1000;
        let conn = self.conn.lock().unwrap();
        match conn.execute("DELETE FROM io_samples WHERE ts < ?1", [cutoff]) {
            Ok(n) if n > 0 => info!("Pruned {n} old metric samples"),
            Err(e) => warn!("Failed to prune metrics: {e}"),
            _ => {}
        }
    }

    /// Query history for a given kind (net/disk) and optional resource name.
    /// Returns last `duration_secs` of data. If `name` is None, returns all resources.
    pub fn query(
        &self,
        kind: &str,
        name: Option<&str>,
        duration_secs: i64,
    ) -> Vec<ResourceHistory> {
        let since = now_ms() - duration_secs * 1000;
        let conn = self.conn.lock().unwrap();

        // Collect distinct resource names
        let names: Vec<String> = if let Some(n) = name {
            vec![n.to_string()]
        } else {
            let mut stmt = conn
                .prepare("SELECT DISTINCT name FROM io_samples WHERE kind = ?1 AND ts >= ?2")
                .unwrap();
            stmt.query_map(rusqlite::params![kind, since], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };

        let mut results = Vec::new();
        let mut stmt = conn
            .prepare(
                "SELECT ts, in_rate, out_rate FROM io_samples
                 WHERE kind = ?1 AND name = ?2 AND ts >= ?3
                 ORDER BY ts",
            )
            .unwrap();

        for n in &names {
            let samples: Vec<IoSample> = stmt
                .query_map(rusqlite::params![kind, n, since], |row| {
                    Ok(IoSample {
                        ts: row.get(0)?,
                        in_rate: row.get(1)?,
                        out_rate: row.get(2)?,
                    })
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            results.push(ResourceHistory {
                name: n.clone(),
                samples,
            });
        }

        results
    }
}

#[derive(Debug, Serialize)]
pub struct ResourceHistory {
    pub name: String,
    pub samples: Vec<IoSample>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
