<script lang="ts">
	import { onMount } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import type { NfsShare } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent } from '$lib/components/ui/card';

	let shares: NfsShare[] = $state([]);
	let showCreate = $state(false);
	let loading = $state(true);

	let newPath = $state('');
	let newComment = $state('');
	let newHost = $state('');
	let newOptions = $state('rw,sync,no_subtree_check');

	const client = getClient();

	onMount(async () => {
		await refresh();
		loading = false;
	});

	async function refresh() {
		await withToast(async () => {
			shares = await client.call<NfsShare[]>('share.nfs.list');
		});
	}

	async function create() {
		if (!newPath || !newHost) return;
		const ok = await withToast(
			() => client.call('share.nfs.create', {
				path: newPath,
				comment: newComment || undefined,
				clients: [{ host: newHost, options: newOptions }],
			}),
			'NFS share created'
		);
		if (ok !== undefined) {
			showCreate = false;
			newPath = '';
			newComment = '';
			newHost = '';
			await refresh();
		}
	}

	async function toggleEnabled(share: NfsShare) {
		await withToast(
			() => client.call('share.nfs.update', { id: share.id, enabled: !share.enabled }),
			`NFS share ${share.enabled ? 'disabled' : 'enabled'}`
		);
		await refresh();
	}

	async function remove(id: string) {
		if (!confirm('Delete this NFS share?')) return;
		await withToast(
			() => client.call('share.nfs.delete', { id }),
			'NFS share deleted'
		);
		await refresh();
	}
</script>

<h1 class="mb-4 text-2xl font-bold">NFS Shares</h1>

<div class="mb-4">
	<Button onclick={() => showCreate = !showCreate}>
		{showCreate ? 'Cancel' : 'Create Share'}
	</Button>
</div>

{#if showCreate}
	<Card class="mb-6 max-w-lg">
		<CardContent class="pt-6">
			<h3 class="mb-4 text-lg font-semibold">New NFS Share</h3>
			<div class="mb-4">
				<Label for="nfs-path">Path</Label>
				<Input id="nfs-path" bind:value={newPath} placeholder="/mnt/nasty/tank/data" class="mt-1" />
			</div>
			<div class="mb-4">
				<Label for="nfs-comment">Comment</Label>
				<Input id="nfs-comment" bind:value={newComment} placeholder="Optional description" class="mt-1" />
			</div>
			<div class="mb-4">
				<Label for="nfs-host">Allowed Network</Label>
				<Input id="nfs-host" bind:value={newHost} placeholder="192.168.1.0/24" class="mt-1" />
			</div>
			<div class="mb-4">
				<Label for="nfs-opts">Options</Label>
				<Input id="nfs-opts" bind:value={newOptions} class="mt-1" />
			</div>
			<Button onclick={create} disabled={!newPath || !newHost}>Create</Button>
		</CardContent>
	</Card>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if shares.length === 0}
	<p class="text-muted-foreground">No NFS shares configured.</p>
{:else}
	<table class="w-full text-sm">
		<thead>
			<tr>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Path</th>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Clients</th>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Status</th>
				<th class="border-b-2 border-border p-3 text-left text-xs uppercase text-muted-foreground">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each shares as share}
				<tr class="border-b border-border">
					<td class="p-3">
						<strong class="font-mono text-sm">{share.path}</strong>
						{#if share.comment}<br /><span class="text-xs text-muted-foreground">{share.comment}</span>{/if}
					</td>
					<td class="p-3">
						{#each share.clients as c}
							<div class="text-sm">{c.host} <span class="text-xs text-muted-foreground">({c.options})</span></div>
						{/each}
					</td>
					<td class="p-3">
						<Badge variant={share.enabled ? 'default' : 'secondary'}>
							{share.enabled ? 'Enabled' : 'Disabled'}
						</Badge>
					</td>
					<td class="p-3">
						<div class="flex gap-2">
							<Button variant="secondary" size="sm" onclick={() => toggleEnabled(share)}>
								{share.enabled ? 'Disable' : 'Enable'}
							</Button>
							<Button variant="destructive" size="sm" onclick={() => remove(share.id)}>Delete</Button>
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/if}
