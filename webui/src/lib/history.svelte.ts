/**
 * Rolling history buffer for dashboard time-series charts.
 * Stores the last N samples of per-resource rates (bytes/s).
 * In the future this will be backed by a time-series DB.
 */

const MAX_SAMPLES = 60; // 5 min at 5s interval

export interface Sample {
	time: Date;
	/** Keyed by resource name (interface/disk), values are rates in bytes/s */
	[key: string]: number | Date;
}

export interface IoRateHistory {
	/** Per-resource history: resource name -> array of {time, in, out} */
	resources: Map<string, { time: Date; in: number; out: number }[]>;
}

function createHistory(): IoRateHistory {
	return { resources: new Map() };
}

function pushSample(
	history: IoRateHistory,
	name: string,
	time: Date,
	inRate: number,
	outRate: number,
) {
	let samples = history.resources.get(name);
	if (!samples) {
		samples = [];
		history.resources.set(name, samples);
	}
	samples.push({ time, in: inRate, out: outRate });
	if (samples.length > MAX_SAMPLES) {
		samples.splice(0, samples.length - MAX_SAMPLES);
	}
}

/**
 * Reactive history store using Svelte 5 runes.
 */
export function createIoHistory() {
	let history = $state<IoRateHistory>(createHistory());

	return {
		get resources() {
			return history.resources;
		},
		push(name: string, time: Date, inRate: number, outRate: number) {
			pushSample(history, name, time, inRate, outRate);
			// Trigger reactivity by reassigning
			history = { resources: new Map(history.resources) };
		},
		getSamples(name: string) {
			return history.resources.get(name) ?? [];
		},
	};
}
