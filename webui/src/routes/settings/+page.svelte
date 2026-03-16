<script lang="ts">
	import { onMount } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import type { Settings, SystemInfo, NetworkConfig } from '$lib/types';
	import { Button } from '$lib/components/ui/button';

	let settings: Settings | null = $state(null);
	let info: SystemInfo | null = $state(null);
	let timezones: string[] = $state([]);
	let saving = $state(false);
	let savingHostname = $state(false);
	let hostnameInput = $state('');

	// Network
	let network: NetworkConfig | null = $state(null);
	let savingNetwork = $state(false);
	let netDhcp = $state(true);
	let netAddress = $state('');
	let netPrefix = $state('24');
	let netGateway = $state('');
	let netNameservers = $state('');
	let netChanged = $state(false);

	const client = getClient();

	onMount(async () => {
		await withToast(async () => {
			[settings, info, timezones, network] = await Promise.all([
				client.call<Settings>('system.settings.get'),
				client.call<SystemInfo>('system.info'),
				client.call<string[]>('system.settings.timezones'),
				client.call<NetworkConfig>('system.network.get'),
			]);
			hostnameInput = settings.hostname ?? info.hostname;
			syncNetworkForm();
		});
	});

	function syncNetworkForm() {
		if (!network) return;
		netDhcp = network.dhcp;
		netAddress = network.address ?? '';
		netPrefix = String(network.prefix_length ?? 24);
		netGateway = network.gateway ?? '';
		netNameservers = network.nameservers.join(', ');
		netChanged = false;
	}

	async function saveHostname() {
		savingHostname = true;
		await withToast(
			() => client.call('system.settings.update', { hostname: hostnameInput }),
			'Hostname updated'
		);
		info = await client.call<SystemInfo>('system.info');
		savingHostname = false;
	}

	async function saveTimezone() {
		if (!settings) return;
		saving = true;
		await withToast(
			() => client.call('system.settings.update', { timezone: settings!.timezone }),
			'Timezone updated'
		);
		info = await client.call<SystemInfo>('system.info');
		saving = false;
	}

	async function saveClock24h(val: boolean) {
		if (!settings) return;
		settings.clock_24h = val;
		await withToast(
			() => client.call('system.settings.update', { clock_24h: val }),
			val ? '24-hour clock enabled' : '12-hour clock enabled'
		);
	}

	async function saveNetwork() {
		savingNetwork = true;
		const nameservers = netNameservers
			.split(/[,\s]+/)
			.map((s) => s.trim())
			.filter(Boolean);
		const payload: Partial<NetworkConfig> = { dhcp: netDhcp, nameservers };
		if (!netDhcp) {
			payload.address = netAddress.trim() || null;
			payload.prefix_length = parseInt(netPrefix) || null;
			payload.gateway = netGateway.trim() || null;
		}
		await withToast(
			() => client.call('system.network.update', payload),
			'Network configuration applied'
		);
		network = await client.call<NetworkConfig>('system.network.get');
		syncNetworkForm();
		savingNetwork = false;
	}
</script>


{#if !settings}
	<p class="text-muted-foreground">Loading...</p>
{:else}
	<div class="grid grid-cols-1 gap-6 xl:grid-cols-2">

		<!-- Left column -->
		<div class="flex flex-col gap-6">

			<!-- System -->
			<section class="rounded-lg border border-border p-5">
				<h2 class="mb-4 text-base font-semibold">System</h2>

				<div class="mb-4 flex items-center justify-between">
					<span class="text-sm text-muted-foreground">Hostname</span>
					<span class="text-sm font-medium font-mono">{info?.hostname ?? '—'}</span>
				</div>

				<div class="flex gap-2">
					<input
						id="hostname"
						type="text"
						bind:value={hostnameInput}
						class="min-w-0 flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
						placeholder="nasty"
					/>
					<Button size="sm" onclick={saveHostname} disabled={savingHostname}>
						{savingHostname ? 'Saving…' : 'Apply'}
					</Button>
				</div>
			</section>

			<!-- Date & Time -->
			<section class="rounded-lg border border-border p-5">
				<h2 class="mb-4 text-base font-semibold">Date & Time</h2>

				<div class="mb-3 flex items-center justify-between">
					<span class="text-sm text-muted-foreground">NTP Synchronization</span>
					<div class="flex items-center gap-1.5">
						<span class="inline-block h-2 w-2 rounded-full {info?.ntp_synced ? 'bg-green-400' : 'bg-yellow-400'}"></span>
						<span class="text-sm">{info?.ntp_synced ? 'Synchronized' : 'Not synchronized'}</span>
					</div>
				</div>

				<div class="mb-3 flex items-center justify-between">
					<span class="text-sm text-muted-foreground">Active Timezone</span>
					<span class="text-sm font-medium font-mono">{info?.timezone ?? '—'}</span>
				</div>

				<div class="mb-4 flex items-center justify-between">
					<span class="text-sm text-muted-foreground">Clock Format</span>
					<div class="flex rounded-md border border-border text-xs">
						<button
							onclick={() => saveClock24h(true)}
							class="rounded-l-md px-3 py-1 font-medium transition-colors {settings.clock_24h ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'}"
						>24h</button>
						<button
							onclick={() => saveClock24h(false)}
							class="rounded-r-md px-3 py-1 font-medium transition-colors {!settings.clock_24h ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'}"
						>AM/PM</button>
					</div>
				</div>

				<div class="flex gap-2">
					<select
						id="timezone"
						bind:value={settings.timezone}
						class="min-w-0 flex-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
					>
						{#each timezones as tz}
							<option value={tz}>{tz}</option>
						{/each}
					</select>
					<Button size="sm" onclick={saveTimezone} disabled={saving}>
						{saving ? 'Saving…' : 'Apply'}
					</Button>
				</div>
			</section>

		</div>

		<!-- Right column: Network -->
		{#if network}
		<section class="rounded-lg border border-border p-5">
			<h2 class="mb-4 text-base font-semibold">Network</h2>

			{#if network.live_addresses.length > 0}
				<div class="mb-4 flex items-start justify-between gap-4">
					<span class="shrink-0 text-sm text-muted-foreground">Active Address</span>
					<div class="text-right">
						<div class="text-sm font-medium font-mono">
							{network.live_addresses.join(', ')}
							{#if network.live_gateway}
								<span class="ml-1 text-muted-foreground">via {network.live_gateway}</span>
							{/if}
						</div>
						<div class="text-xs text-muted-foreground">{network.interface || '—'}</div>
					</div>
				</div>
			{/if}

			<div class="mb-4">
				<div class="mb-2 text-sm text-muted-foreground">Mode</div>
				<div class="flex w-fit rounded-md border border-border text-sm">
					<button
						onclick={() => { netDhcp = true; netChanged = true; }}
						class="rounded-l-md px-4 py-1.5 font-medium transition-colors {netDhcp ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'}"
					>DHCP</button>
					<button
						onclick={() => { netDhcp = false; netChanged = true; }}
						class="rounded-r-md px-4 py-1.5 font-medium transition-colors {!netDhcp ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent'}"
					>Static</button>
				</div>
			</div>

			{#if !netDhcp}
				<div class="mb-4 grid grid-cols-2 gap-3">
					<div>
						<label for="net-address" class="mb-1 block text-xs text-muted-foreground">IP Address</label>
						<input
							id="net-address"
							type="text"
							bind:value={netAddress}
							oninput={() => { netChanged = true; }}
							class="w-full rounded-md border border-input bg-background px-3 py-1.5 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-ring"
							placeholder="192.168.1.100"
						/>
					</div>
					<div>
						<label for="net-prefix" class="mb-1 block text-xs text-muted-foreground">Prefix Length</label>
						<input
							id="net-prefix"
							type="number"
							min="1"
							max="32"
							bind:value={netPrefix}
							oninput={() => { netChanged = true; }}
							class="w-full rounded-md border border-input bg-background px-3 py-1.5 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-ring"
							placeholder="24"
						/>
					</div>
					<div>
						<label for="net-gateway" class="mb-1 block text-xs text-muted-foreground">Gateway</label>
						<input
							id="net-gateway"
							type="text"
							bind:value={netGateway}
							oninput={() => { netChanged = true; }}
							class="w-full rounded-md border border-input bg-background px-3 py-1.5 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-ring"
							placeholder="192.168.1.1"
						/>
					</div>
					<div>
						<label for="net-dns" class="mb-1 block text-xs text-muted-foreground">DNS Servers</label>
						<input
							id="net-dns"
							type="text"
							bind:value={netNameservers}
							oninput={() => { netChanged = true; }}
							class="w-full rounded-md border border-input bg-background px-3 py-1.5 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-ring"
							placeholder="1.1.1.1, 8.8.8.8"
						/>
					</div>
				</div>
			{/if}

			{#if netChanged && !netDhcp}
				<p class="mb-3 text-xs text-amber-500">
					Changing the static IP will move your connection to the new address.
					If it differs from the current one, reconnect to continue.
				</p>
			{/if}

			<Button size="sm" onclick={saveNetwork} disabled={savingNetwork || !netChanged}>
				{savingNetwork ? 'Applying…' : 'Apply Network'}
			</Button>
		</section>
		{/if}

	</div>
{/if}
