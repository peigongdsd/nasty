// Mirrors middleware Rust types

export interface SystemInfo {
	hostname: string;
	version: string;
	uptime_seconds: number;
	kernel: string;
}

export interface SystemHealth {
	status: string;
	services: ServiceStatus[];
}

export interface ServiceStatus {
	name: string;
	running: boolean;
}

export interface Pool {
	name: string;
	uuid: string;
	devices: string[];
	mount_point: string | null;
	mounted: boolean;
	total_bytes: number;
	used_bytes: number;
	available_bytes: number;
	compression: string | null;
	replicas: number;
}

export interface BlockDevice {
	path: string;
	size_bytes: number;
	dev_type: string;
	mount_point: string | null;
	fs_type: string | null;
	in_use: boolean;
}

export interface Subvolume {
	name: string;
	pool: string;
	path: string;
	size_bytes: number | null;
	snapshots: string[];
}

export interface Snapshot {
	name: string;
	subvolume: string;
	pool: string;
	path: string;
	read_only: boolean;
}

export interface NfsShare {
	id: string;
	path: string;
	comment: string | null;
	clients: NfsClient[];
	enabled: boolean;
}

export interface NfsClient {
	host: string;
	options: string;
}

export interface SmbShare {
	id: string;
	name: string;
	path: string;
	comment: string | null;
	read_only: boolean;
	browseable: boolean;
	guest_ok: boolean;
	valid_users: string[];
	extra_params: Record<string, string>;
	enabled: boolean;
}

export interface IscsiTarget {
	id: string;
	iqn: string;
	alias: string | null;
	portals: Portal[];
	luns: Lun[];
	acls: Acl[];
	enabled: boolean;
}

export interface Portal {
	ip: string;
	port: number;
}

export interface Lun {
	lun_id: number;
	backstore_path: string;
	backstore_name: string;
	backstore_type: string;
	size_bytes: number | null;
}

export interface Acl {
	initiator_iqn: string;
	userid: string | null;
	password: string | null;
}

export interface NvmeofSubsystem {
	id: string;
	nqn: string;
	namespaces: Namespace[];
	ports: NvmeofPort[];
	allowed_hosts: string[];
	allow_any_host: boolean;
	enabled: boolean;
}

export interface Namespace {
	nsid: number;
	device_path: string;
	enabled: boolean;
}

export interface NvmeofPort {
	port_id: number;
	transport: string;
	addr: string;
	service_id: string;
	addr_family: string;
}

export interface UserInfo {
	username: string;
	role: 'admin' | 'readonly';
}

export interface SystemStats {
	cpu: CpuStats;
	memory: MemoryStats;
	network: NetIfStats[];
	disk_io: DiskIoStats[];
}

export interface DiskIoStats {
	name: string;
	read_bytes: number;
	write_bytes: number;
	read_ios: number;
	write_ios: number;
	io_in_progress: number;
}

export interface CpuStats {
	count: number;
	load_1: number;
	load_5: number;
	load_15: number;
}

export interface MemoryStats {
	total_bytes: number;
	used_bytes: number;
	available_bytes: number;
	swap_total_bytes: number;
	swap_used_bytes: number;
}

export interface NetIfStats {
	name: string;
	rx_bytes: number;
	tx_bytes: number;
	rx_packets: number;
	tx_packets: number;
	speed_mbps: number | null;
	up: boolean;
}

export interface DiskHealth {
	device: string;
	model: string;
	serial: string;
	firmware: string;
	capacity_bytes: number;
	temperature_c: number | null;
	power_on_hours: number | null;
	health_passed: boolean;
	smart_status: string;
	attributes: SmartAttribute[];
}

export interface SmartAttribute {
	id: number;
	name: string;
	value: number;
	worst: number;
	threshold: number;
	raw_value: number;
	failing: boolean;
}

export interface AlertRule {
	id: string;
	name: string;
	enabled: boolean;
	metric: AlertMetric;
	condition: AlertCondition;
	threshold: number;
	severity: AlertSeverity;
}

export type AlertMetric = 'pool_usage_percent' | 'cpu_load_percent' | 'memory_usage_percent' | 'disk_temperature' | 'smart_health' | 'swap_usage_percent';
export type AlertCondition = 'above' | 'below' | 'equals';
export type AlertSeverity = 'warning' | 'critical';

export interface ActiveAlert {
	rule_id: string;
	rule_name: string;
	severity: AlertSeverity;
	metric: AlertMetric;
	message: string;
	current_value: number;
	threshold: number;
	source: string;
}
