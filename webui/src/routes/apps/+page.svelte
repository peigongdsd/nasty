<script lang="ts">
	import { onMount } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import { confirm } from '$lib/confirm.svelte';
	import type { AppsStatus, App } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent } from '$lib/components/ui/card';
	import SortTh from '$lib/components/SortTh.svelte';

	let status: AppsStatus | null = $state(null);
	let apps: App[] = $state([]);
	let loading = $state(true);
	let enabling = $state(false);
	let showInstall = $state(false);
	let expanded: Record<string, boolean> = $state({});
	let logsApp: string | null = $state(null);
	let logsContent = $state('');

	// Install form
	let newName = $state('');
	let newImage = $state('');
	let newPorts = $state<{ name: string; container_port: number; node_port: string; protocol: string }[]>([]);
	let newEnvs = $state<{ name: string; value: string }[]>([]);
	let newVolumes = $state<{ name: string; mount_path: string; size: string }[]>([]);

	const client = getClient();

	onMount(async () => {
		await refresh();
		loading = false;
	});

	async function refresh() {
		try {
			status = await client.call<AppsStatus>('apps.status');
			if (status.enabled && status.running) {
				apps = await client.call<App[]>('apps.list');
			} else {
				apps = [];
			}
		} catch { /* ignore */ }
	}

	async function enableApps() {
		enabling = true;
		await withToast(
			() => client.call('apps.enable'),
			'Apps runtime enabled'
		);
		enabling = false;
		await refresh();
	}

	async function disableApps() {
		if (!await confirm(
			'Disable apps runtime?',
			'All running apps will be stopped. k3s will be shut down to free memory. App configurations are preserved.'
		)) return;
		await withToast(
			() => client.call('apps.disable'),
			'Apps runtime disabled'
		);
		await refresh();
	}

	function addPort() {
		newPorts = [...newPorts, { name: `port${newPorts.length}`, container_port: 8080, node_port: '', protocol: 'TCP' }];
	}

	function removePort(i: number) {
		newPorts = newPorts.filter((_, idx) => idx !== i);
	}

	function addEnv() {
		newEnvs = [...newEnvs, { name: '', value: '' }];
	}

	function removeEnv(i: number) {
		newEnvs = newEnvs.filter((_, idx) => idx !== i);
	}

	function addVolume() {
		newVolumes = [...newVolumes, { name: `data${newVolumes.length}`, mount_path: '', size: '1Gi' }];
	}

	function removeVolume(i: number) {
		newVolumes = newVolumes.filter((_, idx) => idx !== i);
	}

	async function install() {
		if (!newName || !newImage) return;
		const params: Record<string, unknown> = {
			name: newName,
			image: newImage,
		};
		if (newPorts.length > 0) {
			params.ports = newPorts.map(p => ({
				name: p.name,
				container_port: p.container_port,
				node_port: p.node_port ? parseInt(p.node_port) : undefined,
				protocol: p.protocol,
			}));
		}
		if (newEnvs.length > 0) {
			params.env = newEnvs.filter(e => e.name);
		}
		if (newVolumes.length > 0) {
			params.volumes = newVolumes.filter(v => v.name && v.mount_path);
		}

		const ok = await withToast(
			() => client.call('apps.install', params),
			'App installed'
		);
		if (ok !== undefined) {
			showInstall = false;
			newName = ''; newImage = ''; newPorts = []; newEnvs = []; newVolumes = [];
			await refresh();
		}
	}

	async function removeApp(name: string) {
		if (!await confirm(`Remove app "${name}"?`, 'The app and its resources will be deleted. Persistent volumes may be retained.')) return;
		await withToast(
			() => client.call('apps.remove', { name }),
			'App removed'
		);
		await refresh();
	}

	async function showLogs(name: string) {
		logsApp = name;
		logsContent = 'Loading...';
		try {
			logsContent = await client.call<string>('apps.logs', { name, tail: 200 });
		} catch (e) {
			logsContent = `Failed to load logs: ${e}`;
		}
	}

	function formatMemory(bytes: number): string {
		if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
		if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
		return `${bytes} B`;
	}

	let search = $state('');
	let sortDir = $state<'asc' | 'desc'>('asc');

	function toggleSort() {
		sortDir = sortDir === 'asc' ? 'desc' : 'asc';
	}

	const filtered = $derived(
		search.trim()
			? apps.filter(a => a.name.toLowerCase().includes(search.toLowerCase()))
			: apps
	);

	const sorted = $derived.by(() => {
		return [...filtered].sort((a, b) => {
			const cmp = a.name.localeCompare(b.name);
			return sortDir === 'asc' ? cmp : -cmp;
		});
	});
</script>

<!-- Status Card -->
{#if status}
	<Card class="mb-4">
		<CardContent class="flex items-center gap-4 py-3">
			{#if status.enabled}
				<Badge variant={status.running ? 'default' : 'destructive'}>
					{status.running ? 'Running' : 'Starting...'}
				</Badge>
				<span class="text-sm text-muted-foreground">
					{status.app_count} app{status.app_count !== 1 ? 's' : ''}
					{#if status.memory_bytes}
						&middot; k3s using {formatMemory(status.memory_bytes)}
					{/if}
				</span>
				<Button size="xs" variant="destructive" onclick={disableApps}>
					Disable Apps
				</Button>
			{:else}
				<Badge variant="secondary">Disabled</Badge>
				<span class="text-sm text-muted-foreground">
					App runtime is not running. Enable to deploy containerized applications.
				</span>
			{/if}
		</CardContent>
	</Card>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if !status?.enabled}
	<!-- Enable prompt -->
	<Card class="max-w-lg">
		<CardContent class="pt-6 text-center">
			<h3 class="text-lg font-semibold mb-2">App Runtime</h3>
			<p class="text-sm text-muted-foreground mb-4">
				NASty can run containerized applications using a lightweight Kubernetes runtime (k3s).
				This uses approximately 500 MiB–1 GiB of RAM.
			</p>
			<Button onclick={enableApps} disabled={enabling}>
				{enabling ? 'Enabling...' : 'Enable Apps'}
			</Button>
		</CardContent>
	</Card>
{:else if !status?.running}
	<p class="text-muted-foreground">Waiting for app runtime to start...</p>
{:else}
	<!-- App management -->
	<div class="mb-4 flex items-center gap-3">
		<Button size="sm" onclick={() => showInstall = !showInstall}>
			{showInstall ? 'Cancel' : 'Install App'}
		</Button>
		<Input bind:value={search} placeholder="Search..." class="h-9 w-48" />
	</div>

	{#if showInstall}
		<Card class="mb-6 max-w-xl">
			<CardContent class="pt-6">
				<h3 class="mb-4 text-lg font-semibold">Install App</h3>
				<div class="mb-4">
					<Label for="app-name">App Name</Label>
					<Input id="app-name" bind:value={newName} placeholder="plex" class="mt-1" />
					<span class="mt-1 block text-xs text-muted-foreground">Must be DNS-safe (lowercase, no spaces).</span>
				</div>
				<div class="mb-4">
					<Label for="app-image">Container Image</Label>
					<Input id="app-image" bind:value={newImage} placeholder="lscr.io/linuxserver/plex:latest" class="mt-1" />
				</div>

				<!-- Ports -->
				<div class="mb-4">
					<div class="flex items-center justify-between mb-1">
						<Label>Ports</Label>
						<Button size="xs" variant="outline" onclick={addPort}>+ Add Port</Button>
					</div>
					{#each newPorts as port, i}
						<div class="grid grid-cols-[1fr_80px_90px_60px_auto] gap-2 mt-1 items-center">
							<Input bind:value={port.name} placeholder="Name" class="h-8 text-xs" />
							<Input type="number" bind:value={port.container_port} placeholder="Port" class="h-8 text-xs" />
							<Input bind:value={port.node_port} placeholder="NodePort" class="h-8 text-xs" />
							<select bind:value={port.protocol} class="h-8 rounded-md border border-input bg-transparent px-1 text-xs">
								<option>TCP</option>
								<option>UDP</option>
							</select>
							<Button size="xs" variant="ghost" onclick={() => removePort(i)}>x</Button>
						</div>
					{/each}
				</div>

				<!-- Environment Variables -->
				<div class="mb-4">
					<div class="flex items-center justify-between mb-1">
						<Label>Environment Variables</Label>
						<Button size="xs" variant="outline" onclick={addEnv}>+ Add</Button>
					</div>
					{#each newEnvs as env, i}
						<div class="grid grid-cols-[1fr_1fr_auto] gap-2 mt-1 items-center">
							<Input bind:value={env.name} placeholder="Name" class="h-8 text-xs" />
							<Input bind:value={env.value} placeholder="Value" class="h-8 text-xs" />
							<Button size="xs" variant="ghost" onclick={() => removeEnv(i)}>x</Button>
						</div>
					{/each}
				</div>

				<!-- Volumes -->
				<div class="mb-4">
					<div class="flex items-center justify-between mb-1">
						<Label>Volumes</Label>
						<Button size="xs" variant="outline" onclick={addVolume}>+ Add Volume</Button>
					</div>
					{#each newVolumes as vol, i}
						<div class="grid grid-cols-[1fr_1fr_80px_auto] gap-2 mt-1 items-center">
							<Input bind:value={vol.name} placeholder="Name" class="h-8 text-xs" />
							<Input bind:value={vol.mount_path} placeholder="/config" class="h-8 text-xs" />
							<Input bind:value={vol.size} placeholder="1Gi" class="h-8 text-xs" />
							<Button size="xs" variant="ghost" onclick={() => removeVolume(i)}>x</Button>
						</div>
					{/each}
					{#if newVolumes.length > 0}
						<span class="mt-1 block text-xs text-muted-foreground">Storage provided by nasty-csi via bcachefs subvolumes.</span>
					{/if}
				</div>

				<Button onclick={install} disabled={!newName || !newImage}>Install</Button>
			</CardContent>
		</Card>
	{/if}

	{#if apps.length === 0 && !showInstall}
		<p class="text-muted-foreground">No apps installed.</p>
	{:else if apps.length > 0}
		<table class="w-full text-sm">
			<thead>
				<tr>
					<SortTh label="Name" active={true} dir={sortDir} onclick={toggleSort} />
					<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Chart</th>
					<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Status</th>
					<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground w-px whitespace-nowrap">Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each sorted as app}
					<tr class="border-b border-border hover:bg-muted/30 transition-colors">
						<td class="p-3">
							<span class="font-semibold">{app.name}</span>
						</td>
						<td class="p-3 text-xs text-muted-foreground font-mono">
							{app.chart}
						</td>
						<td class="p-3">
							<Badge variant={app.status === 'deployed' ? 'default' : 'secondary'}>
								{app.status}
							</Badge>
						</td>
						<td class="p-3">
							<div class="flex gap-2">
								<Button variant="outline" size="xs" onclick={() => showLogs(app.name)}>
									Logs
								</Button>
								<Button variant="destructive" size="xs" onclick={() => removeApp(app.name)}>
									Remove
								</Button>
							</div>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
{/if}

<!-- Logs Modal -->
{#if logsApp}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
		<div class="flex flex-col w-[90vw] max-w-4xl h-[70vh] rounded-lg border border-border bg-[#0f1117] shadow-2xl">
			<div class="flex items-center justify-between px-4 py-2 border-b border-border">
				<span class="text-sm font-semibold text-white">Logs: {logsApp}</span>
				<Button variant="ghost" size="xs" onclick={() => logsApp = null} class="text-white hover:text-white/80">
					Close
				</Button>
			</div>
			<pre class="flex-1 p-4 overflow-auto text-xs text-green-400 font-mono whitespace-pre-wrap">{logsContent}</pre>
		</div>
	</div>
{/if}
