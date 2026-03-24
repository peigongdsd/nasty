<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import { confirm } from '$lib/confirm.svelte';
	import type { VmStatus, VmCapabilities, Subvolume } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent } from '$lib/components/ui/card';
	import SortTh from '$lib/components/SortTh.svelte';

	let vms: VmStatus[] = $state([]);
	let capabilities: VmCapabilities | null = $state(null);
	let blockSubvolumes: Subvolume[] = $state([]);
	let showCreate = $state(false);
	let loading = $state(true);
	let expanded: Record<string, boolean> = $state({});
	let actionInProgress: Record<string, boolean> = $state({});

	// Create form
	let newName = $state('');
	let newCpus = $state(1);
	let newMemory = $state(1024);
	let newDisk = $state('');
	let newIso = $state('');
	let newDescription = $state('');
	let newBootOrder = $state('disk');
	let newAutostart = $state(false);

	const client = getClient();

	$effect(() => {
		if (showCreate) loadSubvolumes();
	});

	onMount(async () => {
		await Promise.all([refresh(), loadCapabilities()]);
		loading = false;
	});

	async function refresh() {
		await withToast(async () => {
			vms = await client.call<VmStatus[]>('vm.list');
		});
	}

	async function loadCapabilities() {
		try {
			capabilities = await client.call<VmCapabilities>('vm.capabilities');
		} catch { /* ignore */ }
	}

	async function loadSubvolumes() {
		await withToast(async () => {
			const all = await client.call<Subvolume[]>('subvolume.list_all');
			blockSubvolumes = all.filter(s => s.subvolume_type === 'block' && s.block_device);
		});
	}

	function toggle(id: string) {
		expanded[id] = !expanded[id];
	}

	async function create() {
		if (!newName) return;
		const params: Record<string, unknown> = {
			name: newName,
			cpus: newCpus,
			memory_mib: newMemory,
			boot_order: newBootOrder,
			autostart: newAutostart,
		};
		if (newDisk) {
			params.disks = [{ path: newDisk, interface: 'virtio', readonly: false }];
		}
		if (newIso) {
			params.boot_iso = newIso;
			if (!newDisk) params.boot_order = 'cdrom';
		}
		if (newDescription) params.description = newDescription;

		const ok = await withToast(
			() => client.call('vm.create', params),
			'Virtual machine created'
		);
		if (ok !== undefined) {
			showCreate = false;
			newName = ''; newCpus = 1; newMemory = 1024; newDisk = '';
			newIso = ''; newDescription = ''; newBootOrder = 'disk'; newAutostart = false;
			await refresh();
		}
	}

	async function startVm(id: string) {
		actionInProgress[id] = true;
		await withToast(
			() => client.call('vm.start', { id }),
			'VM started'
		);
		actionInProgress[id] = false;
		await refresh();
	}

	async function stopVm(id: string) {
		actionInProgress[id] = true;
		await withToast(
			() => client.call('vm.stop', { id }),
			'Shutdown signal sent'
		);
		actionInProgress[id] = false;
		// Poll for status change
		setTimeout(() => refresh(), 3000);
	}

	async function killVm(id: string) {
		if (!await confirm('Force kill this VM?', 'This is equivalent to pulling the power cord. Data loss may occur.')) return;
		actionInProgress[id] = true;
		await withToast(
			() => client.call('vm.kill', { id }),
			'VM force-killed'
		);
		actionInProgress[id] = false;
		await refresh();
	}

	async function deleteVm(id: string) {
		if (!await confirm('Delete this virtual machine?', 'The VM configuration will be removed. Disk subvolumes are NOT deleted.')) return;
		await withToast(
			() => client.call('vm.delete', { id }),
			'VM deleted'
		);
		await refresh();
	}

	async function toggleAutostart(vm: VmStatus) {
		await withToast(
			() => client.call('vm.update', { id: vm.id, autostart: !vm.autostart }),
			vm.autostart ? 'Autostart disabled' : 'Autostart enabled'
		);
		await refresh();
	}

	function formatMemory(mib: number): string {
		if (mib >= 1024) return `${(mib / 1024).toFixed(1)} GiB`;
		return `${mib} MiB`;
	}

	let search = $state('');
	let sortDir = $state<'asc' | 'desc'>('asc');

	function toggleSort() {
		sortDir = sortDir === 'asc' ? 'desc' : 'asc';
	}

	const filtered = $derived(
		search.trim()
			? vms.filter(v => v.name.toLowerCase().includes(search.toLowerCase()))
			: vms
	);

	const sorted = $derived.by(() => {
		return [...filtered].sort((a, b) => {
			const cmp = a.name.localeCompare(b.name);
			return sortDir === 'asc' ? cmp : -cmp;
		});
	});
</script>

{#if capabilities}
	<Card class="mb-4">
		<CardContent class="flex items-center gap-4 py-3">
			<Badge variant={capabilities.kvm_available ? 'default' : 'destructive'}>
				{capabilities.kvm_available ? 'KVM Available' : 'No KVM'}
			</Badge>
			<span class="text-sm text-muted-foreground">
				{vms.length} VM{vms.length !== 1 ? 's' : ''}
				&middot; {vms.filter(v => v.running).length} running
				&middot; Arch: <code class="rounded bg-secondary px-1.5 py-0.5 text-xs">{capabilities.arch}</code>
			</span>
			{#if capabilities.uefi_available}
				<Badge variant="secondary">UEFI</Badge>
			{/if}
			{#if capabilities.passthrough_devices.length > 0}
				<Badge variant="secondary">{capabilities.passthrough_devices.length} PCI devices</Badge>
			{/if}
		</CardContent>
	</Card>
{/if}

<div class="mb-4 flex items-center gap-3">
	<Button size="sm" onclick={() => showCreate = !showCreate}>
		{showCreate ? 'Cancel' : 'Create VM'}
	</Button>
	<Input bind:value={search} placeholder="Search..." class="h-9 w-48" />
</div>

{#if showCreate}
	<Card class="mb-6 max-w-lg">
		<CardContent class="pt-6">
			<h3 class="mb-4 text-lg font-semibold">New Virtual Machine</h3>
			<div class="mb-4">
				<Label for="vm-name">Name</Label>
				<Input id="vm-name" bind:value={newName} placeholder="my-vm" class="mt-1" />
			</div>
			<div class="grid grid-cols-2 gap-4 mb-4">
				<div>
					<Label for="vm-cpus">CPU Cores</Label>
					<Input id="vm-cpus" type="number" bind:value={newCpus} min={1} max={64} class="mt-1" />
				</div>
				<div>
					<Label for="vm-memory">Memory (MiB)</Label>
					<Input id="vm-memory" type="number" bind:value={newMemory} min={128} step={128} class="mt-1" />
				</div>
			</div>
			<div class="mb-4">
				<Label for="vm-disk">Boot Disk (block subvolume)</Label>
				<select id="vm-disk" bind:value={newDisk} class="mt-1 h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm">
					<option value="">None (ISO boot only)</option>
					{#each blockSubvolumes as sv}
						<option value={sv.block_device}>{sv.filesystem}/{sv.name} ({sv.block_device})</option>
					{/each}
				</select>
			</div>
			<div class="mb-4">
				<Label for="vm-iso">Boot ISO (optional)</Label>
				<Input id="vm-iso" bind:value={newIso} placeholder="/storage/tank/isos/ubuntu.iso" class="mt-1" />
				<span class="mt-1 block text-xs text-muted-foreground">Path to an ISO image for OS installation.</span>
			</div>
			<div class="mb-4">
				<Label for="vm-boot">Boot Order</Label>
				<select id="vm-boot" bind:value={newBootOrder} class="mt-1 h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm">
					<option value="disk">Disk</option>
					<option value="cdrom">CD-ROM (ISO)</option>
					<option value="network">Network (PXE)</option>
				</select>
			</div>
			<div class="mb-4">
				<Label for="vm-desc">Description</Label>
				<Input id="vm-desc" bind:value={newDescription} placeholder="Optional description" class="mt-1" />
			</div>
			<div class="mb-4 flex items-center gap-2">
				<input id="vm-autostart" type="checkbox" bind:checked={newAutostart} class="rounded border-input" />
				<Label for="vm-autostart">Auto-start on NASty boot</Label>
			</div>
			<Button onclick={create} disabled={!newName}>Create</Button>
		</CardContent>
	</Card>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if vms.length === 0}
	<p class="text-muted-foreground">No virtual machines configured.</p>
{:else}
	<table class="w-full text-sm">
		<thead>
			<tr>
				<SortTh label="Name" active={true} dir={sortDir} onclick={toggleSort} />
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Resources</th>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Status</th>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground w-px whitespace-nowrap">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each sorted as vm}
				<tr class="border-b border-border cursor-pointer hover:bg-muted/30 transition-colors" onclick={() => toggle(vm.id)}>
					<td class="p-3">
						<span class="font-semibold">{vm.name}</span>
						{#if vm.autostart}
							<Badge variant="secondary" class="ml-2 text-[0.6rem]">autostart</Badge>
						{/if}
						{#if vm.description}
							<span class="ml-2 text-xs text-muted-foreground">{vm.description}</span>
						{/if}
					</td>
					<td class="p-3 text-xs text-muted-foreground">
						{vm.cpus} vCPU{vm.cpus !== 1 ? 's' : ''}
						&middot; {formatMemory(vm.memory_mib)}
						&middot; {vm.disks.length} disk{vm.disks.length !== 1 ? 's' : ''}
					</td>
					<td class="p-3">
						<Badge variant={vm.running ? 'default' : 'secondary'}>
							{vm.running ? 'Running' : 'Stopped'}
						</Badge>
						{#if vm.pid}
							<span class="ml-1 text-xs text-muted-foreground">PID {vm.pid}</span>
						{/if}
					</td>
					<td class="p-3" onclick={(e) => e.stopPropagation()}>
						<div class="flex gap-2">
							{#if vm.running}
								<Button variant="secondary" size="xs" onclick={() => stopVm(vm.id)} disabled={actionInProgress[vm.id]}>
									Stop
								</Button>
								<Button variant="destructive" size="xs" onclick={() => killVm(vm.id)} disabled={actionInProgress[vm.id]}>
									Kill
								</Button>
							{:else}
								<Button variant="default" size="xs" onclick={() => startVm(vm.id)} disabled={actionInProgress[vm.id]}>
									Start
								</Button>
								<Button variant="destructive" size="xs" onclick={() => deleteVm(vm.id)}>
									Delete
								</Button>
							{/if}
						</div>
					</td>
				</tr>
				{#if expanded[vm.id]}
					<tr class="border-b border-border bg-secondary/20">
						<td colspan="4" class="px-4 py-4">
							<div class="space-y-4">
								<!-- Configuration -->
								<div>
									<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Configuration</h4>
									<div class="grid grid-cols-2 gap-x-8 gap-y-1 text-sm">
										<div><span class="text-muted-foreground">CPU:</span> {vm.cpus} core{vm.cpus !== 1 ? 's' : ''}</div>
										<div><span class="text-muted-foreground">Memory:</span> {formatMemory(vm.memory_mib)}</div>
										<div><span class="text-muted-foreground">Boot:</span> {vm.boot_order}</div>
										<div><span class="text-muted-foreground">UEFI:</span> {vm.uefi ? 'Yes' : 'No (Legacy BIOS)'}</div>
										{#if vm.boot_iso}
											<div class="col-span-2"><span class="text-muted-foreground">ISO:</span> <code class="text-xs">{vm.boot_iso}</code></div>
										{/if}
									</div>
								</div>

								<!-- Disks -->
								<div>
									<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Disks</h4>
									{#if vm.disks.length === 0}
										<p class="text-xs text-muted-foreground">No disks attached</p>
									{:else}
										<div class="space-y-1">
											{#each vm.disks as disk, i}
												<div class="flex items-center gap-3 rounded bg-secondary/50 px-2 py-1.5">
													<span class="font-mono text-xs font-semibold">Disk {i}</span>
													<span class="text-xs text-muted-foreground">{disk.path}</span>
													<Badge variant="secondary" class="text-[0.6rem]">{disk.interface}</Badge>
													{#if disk.readonly}
														<Badge variant="secondary" class="text-[0.6rem]">readonly</Badge>
													{/if}
												</div>
											{/each}
										</div>
									{/if}
								</div>

								<!-- Networks -->
								<div>
									<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Networks</h4>
									{#if vm.networks.length === 0}
										<p class="text-xs text-muted-foreground">No network interfaces</p>
									{:else}
										<div class="space-y-1">
											{#each vm.networks as net, i}
												<div class="flex items-center gap-3 rounded bg-secondary/50 px-2 py-1.5">
													<span class="font-mono text-xs font-semibold">NIC {i}</span>
													<Badge variant="secondary" class="text-[0.6rem]">{net.mode}</Badge>
													{#if net.bridge}
														<span class="text-xs text-muted-foreground">bridge: {net.bridge}</span>
													{/if}
													{#if net.mac}
														<span class="font-mono text-xs text-muted-foreground">{net.mac}</span>
													{/if}
												</div>
											{/each}
										</div>
									{/if}
								</div>

								<!-- Passthrough -->
								{#if vm.passthrough_devices.length > 0}
									<div>
										<h4 class="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">PCI Passthrough</h4>
										<div class="space-y-1">
											{#each vm.passthrough_devices as dev}
												<div class="flex items-center gap-3 rounded bg-secondary/50 px-2 py-1.5">
													<span class="font-mono text-xs">{dev.address}</span>
													{#if dev.label}
														<span class="text-xs text-muted-foreground">{dev.label}</span>
													{/if}
												</div>
											{/each}
										</div>
									</div>
								{/if}

								<!-- Actions -->
								<div class="flex gap-2 pt-2">
									<Button size="xs" variant="outline" onclick={() => toggleAutostart(vm)}>
										{vm.autostart ? 'Disable Autostart' : 'Enable Autostart'}
									</Button>
								</div>
							</div>
						</td>
					</tr>
				{/if}
			{/each}
		</tbody>
	</table>
{/if}
