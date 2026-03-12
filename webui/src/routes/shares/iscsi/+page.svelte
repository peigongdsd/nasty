<script lang="ts">
	import { onMount } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import type { IscsiTarget } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent } from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';

	let targets: IscsiTarget[] = $state([]);
	let showCreate = $state(false);
	let loading = $state(true);

	let newName = $state('');

	let lunTarget = $state<string | null>(null);
	let lunPath = $state('');
	let lunType = $state('block');

	const client = getClient();

	onMount(async () => {
		await refresh();
		loading = false;
	});

	async function refresh() {
		await withToast(async () => {
			targets = await client.call<IscsiTarget[]>('share.iscsi.list');
		});
	}

	async function create() {
		if (!newName) return;
		const ok = await withToast(
			() => client.call('share.iscsi.create', { name: newName }),
			'iSCSI target created'
		);
		if (ok !== undefined) {
			showCreate = false;
			newName = '';
			await refresh();
		}
	}

	async function remove(id: string) {
		if (!confirm('Delete this iSCSI target and all its LUNs?')) return;
		await withToast(
			() => client.call('share.iscsi.delete', { id }),
			'iSCSI target deleted'
		);
		await refresh();
	}

	async function addLun() {
		if (!lunTarget || !lunPath) return;
		const ok = await withToast(
			() => client.call('share.iscsi.add_lun', {
				target_id: lunTarget,
				backstore_path: lunPath,
				backstore_type: lunType,
			}),
			'LUN added'
		);
		if (ok !== undefined) {
			lunTarget = null;
			lunPath = '';
			await refresh();
		}
	}

	async function removeLun(targetId: string, lunId: number) {
		if (!confirm(`Remove LUN ${lunId}?`)) return;
		await withToast(
			() => client.call('share.iscsi.remove_lun', { target_id: targetId, lun_id: lunId }),
			'LUN removed'
		);
		await refresh();
	}
</script>

<h1 class="mb-4 text-2xl font-bold">iSCSI Targets</h1>

<div class="mb-4">
	<Button onclick={() => showCreate = !showCreate}>
		{showCreate ? 'Cancel' : 'Create Target'}
	</Button>
</div>

{#if showCreate}
	<Card class="mb-6 max-w-lg">
		<CardContent class="pt-6">
			<h3 class="mb-4 text-lg font-semibold">New iSCSI Target</h3>
			<div class="mb-4">
				<Label for="iscsi-name">Name</Label>
				<Input id="iscsi-name" bind:value={newName} placeholder="dbserver" class="mt-1" />
				<span class="mt-1 block text-xs text-muted-foreground">IQN: iqn.2024-01.com.nasty:{newName || '...'}</span>
			</div>
			<Button onclick={create} disabled={!newName}>Create</Button>
		</CardContent>
	</Card>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if targets.length === 0}
	<p class="text-muted-foreground">No iSCSI targets configured.</p>
{:else}
	{#each targets as target}
		<Card class="mb-4">
			<CardContent class="pt-5">
				<div class="mb-4 flex items-start justify-between">
					<div>
						<strong class="font-mono text-sm">{target.iqn}</strong>
						{#if target.alias}<span class="text-muted-foreground"> ({target.alias})</span>{/if}
					</div>
					<div class="flex gap-2">
						<Button variant="secondary" size="sm" onclick={() => { lunTarget = target.id; lunPath = ''; }}>Add LUN</Button>
						<Button variant="destructive" size="sm" onclick={() => remove(target.id)}>Delete</Button>
					</div>
				</div>

				<div class="mb-3">
					<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">Portals</h4>
					{#if target.portals.length === 0}
						<span class="text-sm text-muted-foreground">None</span>
					{:else}
						{#each target.portals as p}
							<span class="mr-1 inline-block rounded bg-secondary px-1.5 py-0.5 text-xs">{p.ip}:{p.port}</span>
						{/each}
					{/if}
				</div>

				<div class="mb-3">
					<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">LUNs</h4>
					{#if target.luns.length === 0}
						<span class="text-sm text-muted-foreground">No LUNs</span>
					{:else}
						<table class="w-full text-sm">
							<thead>
								<tr>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">LUN</th>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">Type</th>
									<th class="p-1.5 text-left text-xs uppercase text-muted-foreground">Path</th>
									<th class="p-1.5"></th>
								</tr>
							</thead>
							<tbody>
								{#each target.luns as lun}
									<tr class="border-b border-border">
										<td class="p-1.5">{lun.lun_id}</td>
										<td class="p-1.5"><span class="rounded bg-secondary px-1.5 py-0.5 text-xs">{lun.backstore_type}</span></td>
										<td class="p-1.5 font-mono text-xs">{lun.backstore_path}</td>
										<td class="p-1.5"><Button variant="destructive" size="sm" onclick={() => removeLun(target.id, lun.lun_id)}>Remove</Button></td>
									</tr>
								{/each}
							</tbody>
						</table>
					{/if}
				</div>

				<div>
					<h4 class="mb-1.5 text-xs uppercase tracking-wide text-muted-foreground">ACLs</h4>
					{#if target.acls.length === 0}
						<span class="text-sm text-muted-foreground">Open (any initiator)</span>
					{:else}
						{#each target.acls as acl}
							<div class="font-mono text-sm">{acl.initiator_iqn}</div>
						{/each}
					{/if}
				</div>
			</CardContent>
		</Card>
	{/each}
{/if}

<Dialog.Root open={lunTarget !== null} onOpenChange={(open) => { if (!open) lunTarget = null; }}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Add LUN</Dialog.Title>
		</Dialog.Header>
		<div class="mb-4">
			<Label for="lun-type">Type</Label>
			<select id="lun-type" bind:value={lunType} class="mt-1 h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm">
				<option value="block">Block Device</option>
				<option value="fileio">File I/O</option>
			</select>
		</div>
		<div class="mb-4">
			<Label for="lun-path">Path</Label>
			<Input id="lun-path" bind:value={lunPath} placeholder={lunType === 'block' ? '/dev/sdb' : '/mnt/nasty/pool/disk.img'} class="mt-1" />
		</div>
		<Dialog.Footer>
			<Button onclick={addLun} disabled={!lunPath}>Add</Button>
			<Button variant="secondary" onclick={() => lunTarget = null}>Cancel</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
