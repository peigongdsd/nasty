<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getClient } from '$lib/client';
	import type { UpsStatus } from '$lib/types';

	const client = getClient();

	let status: UpsStatus | null = $state(null);
	let loading = $state(true);
	let statusInterval: ReturnType<typeof setInterval> | null = null;

	onMount(async () => {
		await refreshStatus();
		loading = false;
		statusInterval = setInterval(refreshStatus, 5000);
	});

	onDestroy(() => {
		if (statusInterval) clearInterval(statusInterval);
	});

	async function refreshStatus() {
		try {
			status = await client.call<UpsStatus>('system.nut.status');
		} catch { /* ignore polling errors */ }
	}

	function statusColor(s: string): string {
		if (s === 'OL' || s.startsWith('OL ')) return 'text-green-500';
		if (s.includes('OB')) return 'text-yellow-500';
		if (s.includes('LB')) return 'text-red-500';
		return 'text-muted-foreground';
	}

	function statusLabel(s: string): string {
		const parts = s.split(' ').map(code => {
			switch (code) {
				case 'OL': return 'Online';
				case 'OB': return 'On Battery';
				case 'LB': return 'Low Battery';
				case 'HB': return 'High Battery';
				case 'RB': return 'Replace Battery';
				case 'CHRG': return 'Charging';
				case 'DISCHRG': return 'Discharging';
				case 'BYPASS': return 'Bypass';
				case 'CAL': return 'Calibrating';
				case 'OFF': return 'Offline';
				case 'OVER': return 'Overloaded';
				case 'TRIM': return 'Trimming';
				case 'BOOST': return 'Boosting';
				case 'FSD': return 'Forced Shutdown';
				default: return code;
			}
		});
		return parts.join(' / ');
	}

	function formatRuntime(seconds: number): string {
		if (seconds >= 3600) {
			const h = Math.floor(seconds / 3600);
			const m = Math.floor((seconds % 3600) / 60);
			return `${h}h ${m}m`;
		}
		const m = Math.floor(seconds / 60);
		const s = seconds % 60;
		return `${m}m ${s}s`;
	}
</script>

<h1 class="mb-6 text-xl font-bold">UPS</h1>

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if !status?.available}
	<p class="text-sm text-muted-foreground">UPS not available. Enable the UPS (NUT) service in <a href="/services" class="underline">Services</a>, then configure it in <a href="/settings" class="underline">Settings &gt; UPS</a>.</p>
{:else}
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
		<div>
			<p class="text-xs text-muted-foreground">Status</p>
			<p class="text-lg font-semibold {statusColor(status.status)}">
				{statusLabel(status.status)}
			</p>
		</div>
		{#if status.battery_charge != null}
			<div>
				<p class="text-xs text-muted-foreground">Battery</p>
				<p class="text-lg font-semibold">{status.battery_charge.toFixed(0)}%</p>
			</div>
		{/if}
		{#if status.battery_runtime != null}
			<div>
				<p class="text-xs text-muted-foreground">Runtime</p>
				<p class="text-lg font-semibold">{formatRuntime(status.battery_runtime)}</p>
			</div>
		{/if}
		{#if status.ups_load != null}
			<div>
				<p class="text-xs text-muted-foreground">Load</p>
				<p class="text-lg font-semibold">{status.ups_load.toFixed(0)}%</p>
			</div>
		{/if}
		{#if status.input_voltage != null}
			<div>
				<p class="text-xs text-muted-foreground">Input Voltage</p>
				<p class="text-sm">{status.input_voltage.toFixed(1)} V</p>
			</div>
		{/if}
		{#if status.output_voltage != null}
			<div>
				<p class="text-xs text-muted-foreground">Output Voltage</p>
				<p class="text-sm">{status.output_voltage.toFixed(1)} V</p>
			</div>
		{/if}
		{#if status.ups_model}
			<div>
				<p class="text-xs text-muted-foreground">Model</p>
				<p class="text-sm">{status.ups_model}</p>
			</div>
		{/if}
		{#if status.ups_serial}
			<div>
				<p class="text-xs text-muted-foreground">Serial</p>
				<p class="text-sm font-mono">{status.ups_serial}</p>
			</div>
		{/if}
	</div>

	<p class="mt-4 text-xs text-muted-foreground">Configuration: <a href="/settings" class="underline">Settings &gt; UPS</a></p>
{/if}
