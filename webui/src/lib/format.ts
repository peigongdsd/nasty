export function formatBytes(bytes: number): string {
	if (bytes === 0) return '0 B';
	const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB'];
	const i = Math.floor(Math.log(bytes) / Math.log(1024));
	const val = bytes / Math.pow(1024, i);
	return `${val.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

// Returns a formatter that always uses the unit determined by `maxBytes`,
// so all Y-axis ticks on a chart share the same unit.
export function makeBytesFormatter(maxBytes: number): (v: number) => string {
	const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB'];
	const i = maxBytes > 0 ? Math.floor(Math.log(maxBytes) / Math.log(1024)) : 0;
	const divisor = Math.pow(1024, i);
	const unit = units[i];
	return (v: number) => `${(v / divisor).toFixed(i > 0 ? 1 : 0)} ${unit}`;
}

export function formatUptime(seconds: number): string {
	const days = Math.floor(seconds / 86400);
	const hours = Math.floor((seconds % 86400) / 3600);
	const mins = Math.floor((seconds % 3600) / 60);
	if (days > 0) return `${days}d ${hours}h ${mins}m`;
	if (hours > 0) return `${hours}h ${mins}m`;
	return `${mins}m`;
}

export function formatPercent(used: number, total: number): string {
	if (total === 0) return '0%';
	return `${((used / total) * 100).toFixed(1)}%`;
}
