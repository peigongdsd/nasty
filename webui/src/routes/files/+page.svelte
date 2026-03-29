<script lang="ts">
	import { onMount } from 'svelte';
	import { getToken } from '$lib/auth';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { FolderOpen, File, ArrowUp, Upload, FolderPlus, Trash2 } from '@lucide/svelte';

	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		modified: number;
	}

	let currentPath = $state('');
	let entries: FileEntry[] = $state([]);
	let loading = $state(true);

	// Upload state
	let uploading = $state(false);
	let uploadProgress = $state(0);
	let uploadName = $state('');

	// Mkdir state
	let showMkdir = $state(false);
	let newDirName = $state('');

	// Delete confirmation
	let deleteTarget: FileEntry | null = $state(null);

	onMount(() => browse(''));

	async function browse(path: string) {
		loading = true;
		try {
			const token = getToken();
			const res = await fetch(`/api/files/browse?path=${encodeURIComponent(path)}`, {
				headers: { 'Authorization': `Bearer ${token}` },
			});
			const data = await res.json();
			if (res.ok) {
				currentPath = data.path || '';
				entries = data.entries || [];
			}
		} catch { /* ignore */ }
		loading = false;
	}

	function navigateTo(entry: FileEntry) {
		if (entry.is_dir) {
			browse(currentPath ? `${currentPath}/${entry.name}` : entry.name);
		}
	}

	function goUp() {
		const parts = currentPath.split('/').filter(Boolean);
		parts.pop();
		browse(parts.join('/'));
	}

	function formatSize(bytes: number): string {
		if (bytes === 0) return '—';
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
	}

	function formatDate(epoch: number): string {
		if (!epoch) return '—';
		return new Date(epoch * 1000).toLocaleString();
	}

	const breadcrumbs = $derived.by(() => {
		const parts = currentPath.split('/').filter(Boolean);
		return parts.map((part, i) => ({
			label: part,
			path: parts.slice(0, i + 1).join('/'),
		}));
	});

	// True for top-level dirs (filesystems) — don't allow deleting them here
	const isRoot = $derived(currentPath.split('/').filter(Boolean).length === 0);

	function triggerUpload() {
		const input = document.createElement('input');
		input.type = 'file';
		input.multiple = true;
		input.onchange = async () => {
			if (!input.files?.length) return;
			for (const file of input.files) {
				await uploadFile(file);
			}
		};
		input.click();
	}

	async function uploadFile(file: globalThis.File) {
		uploading = true;
		uploadProgress = 0;
		uploadName = file.name;

		const token = getToken();
		const form = new FormData();
		form.append('file', file);

		try {
			await new Promise<void>((resolve, reject) => {
				const xhr = new XMLHttpRequest();
				xhr.open('POST', `/api/files/upload?path=${encodeURIComponent(currentPath)}`);
				xhr.setRequestHeader('Authorization', `Bearer ${token}`);
				xhr.upload.onprogress = (e) => {
					if (e.lengthComputable) uploadProgress = Math.round((e.loaded / e.total) * 100);
				};
				xhr.onload = () => {
					if (xhr.status === 200) resolve();
					else reject(new Error(JSON.parse(xhr.responseText)?.error || 'Upload failed'));
				};
				xhr.onerror = () => reject(new Error('Network error'));
				xhr.send(form);
			});
		} catch (e: unknown) {
			alert(e instanceof Error ? e.message : 'Upload failed');
		}

		uploading = false;
		uploadProgress = 0;
		uploadName = '';
		await browse(currentPath);
	}

	async function createDir() {
		if (!newDirName.trim()) return;
		const path = currentPath ? `${currentPath}/${newDirName.trim()}` : newDirName.trim();
		const token = getToken();
		const res = await fetch(`/api/files/mkdir?path=${encodeURIComponent(path)}`, {
			method: 'POST',
			headers: { 'Authorization': `Bearer ${token}` },
		});
		if (!res.ok) {
			const data = await res.json();
			alert(data.error || 'Failed to create directory');
			return;
		}
		showMkdir = false;
		newDirName = '';
		await browse(currentPath);
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		const path = currentPath ? `${currentPath}/${deleteTarget.name}` : deleteTarget.name;
		const token = getToken();
		const res = await fetch(`/api/files?path=${encodeURIComponent(path)}`, {
			method: 'DELETE',
			headers: { 'Authorization': `Bearer ${token}` },
		});
		if (!res.ok) {
			const data = await res.json();
			alert(data.error || 'Failed to delete');
		}
		deleteTarget = null;
		await browse(currentPath);
	}
</script>

<!-- Toolbar -->
<div class="mb-4 flex items-center justify-between gap-4">
	<div class="flex items-center gap-1 text-sm font-mono">
		<button class="text-muted-foreground hover:text-foreground transition-colors" onclick={() => browse('')}>
			/fs
		</button>
		{#each breadcrumbs as crumb}
			<span class="text-muted-foreground/50">/</span>
			<button class="text-muted-foreground hover:text-foreground transition-colors" onclick={() => browse(crumb.path)}>
				{crumb.label}
			</button>
		{/each}
	</div>
	<div class="flex items-center gap-2">
		{#if currentPath}
			<Button variant="outline" size="sm" onclick={goUp}>
				<ArrowUp size={14} class="mr-1" /> Up
			</Button>
		{/if}
		{#if !isRoot}
			<Button variant="outline" size="sm" onclick={triggerUpload} disabled={uploading}>
				<Upload size={14} class="mr-1" /> Upload
			</Button>
		{/if}
		{#if !isRoot}
			<Button variant="outline" size="sm" onclick={() => { showMkdir = true; newDirName = ''; }}>
				<FolderPlus size={14} class="mr-1" /> New Folder
			</Button>
		{/if}
	</div>
</div>

<!-- Upload progress -->
{#if uploading}
	<div class="mb-4 rounded-md border border-border p-3">
		<div class="mb-1 flex items-center justify-between text-sm">
			<span class="text-muted-foreground">Uploading <span class="font-mono">{uploadName}</span></span>
			<span class="tabular-nums">{uploadProgress}%</span>
		</div>
		<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
			<div class="h-full rounded-full bg-primary transition-all" style="width: {uploadProgress}%"></div>
		</div>
	</div>
{/if}

<!-- New folder inline -->
{#if showMkdir}
	<div class="mb-4 flex items-center gap-2">
		<input type="text" bind:value={newDirName} placeholder="Folder name"
			class="h-9 w-64 rounded-md border border-input bg-transparent px-3 text-sm"
			onkeydown={(e) => { if (e.key === 'Enter') createDir(); if (e.key === 'Escape') showMkdir = false; }} />
		<Button size="sm" onclick={createDir} disabled={!newDirName.trim()}>Create</Button>
		<Button variant="secondary" size="sm" onclick={() => showMkdir = false}>Cancel</Button>
	</div>
{/if}

<!-- File listing -->
{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else if entries.length === 0}
	<Card>
		<CardContent class="py-8 text-center text-muted-foreground">
			{currentPath ? 'Empty directory' : 'No filesystems mounted'}
		</CardContent>
	</Card>
{:else}
	<table class="w-full text-sm">
		<thead>
			<tr class="border-b-2 border-border">
				<th class="p-3 text-left text-xs uppercase text-muted-foreground">Name</th>
				<th class="p-3 text-right text-xs uppercase text-muted-foreground">Size</th>
				<th class="p-3 text-right text-xs uppercase text-muted-foreground">Modified</th>
				{#if !isRoot}
					<th class="w-10"></th>
				{/if}
			</tr>
		</thead>
		<tbody>
			{#each entries as entry}
				<tr class="border-b border-border hover:bg-muted/30 transition-colors group">
					<td class="p-3">
						{#if entry.is_dir}
							<button class="flex items-center gap-2 hover:text-primary transition-colors" onclick={() => navigateTo(entry)}>
								<FolderOpen size={16} class="text-yellow-500 shrink-0" />
								<span class="font-medium">{entry.name}</span>
							</button>
						{:else}
							<div class="flex items-center gap-2">
								<File size={16} class="text-muted-foreground shrink-0" />
								<span>{entry.name}</span>
							</div>
						{/if}
					</td>
					<td class="p-3 text-right text-muted-foreground tabular-nums">{entry.is_dir ? '—' : formatSize(entry.size)}</td>
					<td class="p-3 text-right text-muted-foreground text-xs tabular-nums">{formatDate(entry.modified)}</td>
					{#if !isRoot}
						<td class="p-3 text-right">
							<button
								class="text-muted-foreground/40 hover:text-destructive transition-colors opacity-0 group-hover:opacity-100"
								onclick={() => deleteTarget = entry}
								title="Delete">
								<Trash2 size={14} />
							</button>
						</td>
					{/if}
				</tr>
			{/each}
		</tbody>
	</table>
{/if}

<!-- Delete confirmation -->
{#if deleteTarget}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
		<Card class="w-full max-w-sm">
			<CardContent class="pt-6">
				<h3 class="mb-2 text-lg font-semibold">Delete {deleteTarget.is_dir ? 'folder' : 'file'}</h3>
				<p class="mb-4 text-sm text-muted-foreground">
					{#if deleteTarget.is_dir}
						Delete <span class="font-mono font-medium text-foreground">{deleteTarget.name}</span> and all its contents? This cannot be undone.
					{:else}
						Delete <span class="font-mono font-medium text-foreground">{deleteTarget.name}</span>? This cannot be undone.
					{/if}
				</p>
				<div class="flex gap-2">
					<Button variant="destructive" onclick={confirmDelete}>Delete</Button>
					<Button variant="secondary" onclick={() => deleteTarget = null}>Cancel</Button>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}
