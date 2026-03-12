<script lang="ts">
	import { onMount } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import type { NvmeofSubsystem } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent } from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';

	let subsystems: NvmeofSubsystem[] = $state([]);
	let showCreate = $state(false);
	let loading = $state(true);

	let newName = $state('');

	let nsSubsys = $state<string | null>(null);
	let nsDevicePath = $state('');

	let portSubsys = $state<string | null>(null);
	let portTransport = $state('tcp');
	let portAddr = $state('0.0.0.0');
	let portSvcId = $state(4420);

	const client = getClient();

	onMount(async () => {
		await refresh();
		loading = false;
	});

	async function refresh() {
		await withToast(async () => {
			subsystems = await client.call<NvmeofSubsystem[]>('share.nvmeof.list');
		});
	}

	async function create() {
		if (!newName) return;
		const ok = await withToast(
			() => client.call('share.nvmeof.create', { name: newName }),
			'NVMe-oF subsystem created'
		);
		if (ok !== undefined) {
			showCreate = false;
			newName = '';
			await refresh();
		}
	}

	async function remove(id: string) {
		if (!confirm('Delete this NVMe-oF subsystem?')) return;
		await withToast(
			() => client.call('share.nvmeof.delete', { id }),
			'NVMe-oF subsystem deleted'
		);
		await refresh();
	}

	async function addNamespace() {
		if (!nsSubsys || !nsDevicePath) return;
		const ok = await withToast(
			() => client.call('share.nvmeof.add_namespace', {
				subsystem_id: nsSubsys,
				device_path: nsDevicePath,
			}),
			'Namespace added'
		);
		if (ok !== undefined) {
			nsSubsys = null;
			nsDevicePath = '';
			await refresh();
		}
	}

	async function removeNamespace(subsysId: string, nsid: number) {
		if (!confirm(`Remove namespace ${nsid}?`)) return;
		await withToast(
			() => client.call('share.nvmeof.remove_namespace', { subsystem_id: subsysId, nsid }),
			'Namespace removed'
		);
		await refresh();
	}

	async function addPort() {
		if (!portSubsys) return;
		const ok = await withToast(
			() => client.call('share.nvmeof.add_port', {
				subsystem_id: portSubsys,
				transport: portTransport,
				addr: portAddr,
				service_id: portSvcId,
			}),
			'Port added'
		);
		if (ok !== undefined) {
			portSubsys = null;
			await refresh();
		}
	}

	async function removePort(subsysId: string, portId: number) {
		if (!confirm(`Remove port ${portId}?`)) return;
		await withToast(
			() => client.call('share.nvmeof.remove_port', { subsystem_id: subsysId, port_id: portId }),
			'Port removed'
		);
		await refresh();
	}
</script>

<h1 class="mb-4 text-2xl font-bold">NVMe-oF Subsystems</h1>

<div class="mb-4">
	<Button onclick={() => showCreate = !showCreate}>
		{showCreate ? 'Cancel' : 'Create Subsystem'}
	</Button>
</div>

{#if showCreate}
	<Card class="mb-6 max-w-lg">
		<CardContent class="pt-6">
			<h3 class="mb-4 text-lg font-semibold">New NVMe-oF Subsystem</h3>
			<div class="mb-4">
				<Label for="nvme-name">Name</Label>
				<Input id="nvme-name" bind:value={newName} placeholder="faststore" class="mt-1" />
				<span class="mt-1 block text-xs text-muted-foreground">NQN: nqn.2024-01.com.nasty:{newName || '...'}</span>
			</div>
			<Button onclick={create} disabled={!newName}>Create</Button>
		</CardContent>
	</Card>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if subsystems.length === 0}
	<p class="text-muted-foreground">No NVMe-oF subsystems configured.</p>
{:else}
	{#each subsystems as subsys}
		<Card class="mb-4">
			<CardContent class="pt-5">
				<div class="mb-4 flex items-start justify-between">
					<div>
						<strong class="font-mono text-sm">{subsys.nqn}</strong>
						<div class="text-xs text-muted-foreground">
							{subsys.allow_any_host ? 'Any host allowed' : `${subsys.allowed_hosts.length} allowed host(s)`}
						</div>
					</div>
					<div class="flex gap-2">
						<Button variant="secondary" size="sm" onclick={() => { nsSubsys = subsys.id; nsDevicePath = ''; }}>Add Namespace</Button>
						<Button variant="secondary" size="sm" onclick={() => { portSubsys = subsys.id; portAddr = '0.0.0.0'; portSvcId = 4420; }}>Add Port</Button>
						<Button variant="destructive" size="sm" onclick={() => remove(subsys.id)}>Delete</Button>
					</div>
				</div>

				<div class="mb-3">
					<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Namespaces</h4>
					{#if subsys.namespaces.length === 0}
						<span class="text-sm text-muted-foreground">No namespaces</span>
					{:else}
						<table class="w-full text-sm">
							<thead>
								<tr>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">NSID</th>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">Device</th>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">Status</th>
									<th class="p-1.5"></th>
								</tr>
							</thead>
							<tbody>
								{#each subsys.namespaces as ns}
									<tr class="border-b border-border">
										<td class="p-1.5">{ns.nsid}</td>
										<td class="p-1.5 font-mono text-xs">{ns.device_path}</td>
										<td class="p-1.5">
											<Badge variant={ns.enabled ? 'default' : 'secondary'}>{ns.enabled ? 'Enabled' : 'Disabled'}</Badge>
										</td>
										<td class="p-1.5"><Button variant="destructive" size="sm" onclick={() => removeNamespace(subsys.id, ns.nsid)}>Remove</Button></td>
									</tr>
								{/each}
							</tbody>
						</table>
					{/if}
				</div>

				<div class="mb-3">
					<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Ports</h4>
					{#if subsys.ports.length === 0}
						<span class="text-sm text-muted-foreground">No ports (not listening)</span>
					{:else}
						{#each subsys.ports as port}
							<div class="my-1 flex items-center gap-2">
								<span class="rounded bg-secondary px-1.5 py-0.5 text-xs">{port.transport.toUpperCase()}</span>
								<span class="font-mono text-sm">{port.addr}:{port.service_id}</span>
								<span class="text-xs text-muted-foreground">({port.addr_family})</span>
								<Button variant="destructive" size="sm" onclick={() => removePort(subsys.id, port.port_id)}>Remove</Button>
							</div>
						{/each}
					{/if}
				</div>

				{#if subsys.allowed_hosts.length > 0}
					<div>
						<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Allowed Hosts</h4>
						{#each subsys.allowed_hosts as host}
							<div class="font-mono text-sm">{host}</div>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>
	{/each}
{/if}

<Dialog.Root open={nsSubsys !== null} onOpenChange={(open) => { if (!open) nsSubsys = null; }}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Add Namespace</Dialog.Title>
		</Dialog.Header>
		<div class="mb-4">
			<Label for="ns-device">Device Path</Label>
			<Input id="ns-device" bind:value={nsDevicePath} placeholder="/dev/nvme0n1" class="mt-1" />
		</div>
		<Dialog.Footer>
			<Button onclick={addNamespace} disabled={!nsDevicePath}>Add</Button>
			<Button variant="secondary" onclick={() => nsSubsys = null}>Cancel</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root open={portSubsys !== null} onOpenChange={(open) => { if (!open) portSubsys = null; }}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Add Port</Dialog.Title>
		</Dialog.Header>
		<div class="mb-4">
			<Label for="port-transport">Transport</Label>
			<select id="port-transport" bind:value={portTransport} class="mt-1 h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm">
				<option value="tcp">TCP</option>
				<option value="rdma">RDMA</option>
			</select>
		</div>
		<div class="mb-4">
			<Label for="port-addr">Address</Label>
			<Input id="port-addr" bind:value={portAddr} class="mt-1" />
		</div>
		<div class="mb-4">
			<Label for="port-svcid">Port</Label>
			<Input id="port-svcid" type="number" bind:value={portSvcId} class="mt-1" />
		</div>
		<Dialog.Footer>
			<Button onclick={addPort}>Add</Button>
			<Button variant="secondary" onclick={() => portSubsys = null}>Cancel</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
