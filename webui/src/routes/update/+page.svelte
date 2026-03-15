<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getClient } from '$lib/client';
	import { withToast } from '$lib/toast.svelte';
	import type { UpdateInfo, UpdateStatus } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent } from '$lib/components/ui/card';

	let info: UpdateInfo | null = $state(null);
	let status: UpdateStatus | null = $state(null);
	let loading = $state(true);
	let checking = $state(false);
	let needsRefresh = $state(false);
	let confirmAction: 'update' | 'rollback' | null = $state(null);
	let confirmTimer: ReturnType<typeof setTimeout> | null = null;
	let pollInterval: ReturnType<typeof setInterval> | null = $state(null);
	let logEl: HTMLPreElement | undefined = $state();

	const client = getClient();

	$effect(() => {
		if (status?.log && logEl) {
			logEl.scrollTop = logEl.scrollHeight;
		}
	});

	onMount(async () => {
		await loadVersion();
		await loadStatus();
		loading = false;
	});

	onDestroy(() => {
		stopPolling();
		if (confirmTimer) clearTimeout(confirmTimer);
	});

	async function loadVersion() {
		await withToast(async () => {
			info = await client.call<UpdateInfo>('system.update.version');
		});
	}

	async function loadStatus() {
		await withToast(async () => {
			status = await client.call<UpdateStatus>('system.update.status');
			if (status?.state === 'running') {
				startPolling();
			}
		});
	}

	async function checkForUpdates() {
		checking = true;
		const result = await withToast(
			() => client.call<UpdateInfo>('system.update.check'),
			'Update check complete'
		);
		if (result !== undefined) {
			info = result;
		}
		checking = false;
	}

	function requestAction(action: 'update' | 'rollback') {
		if (confirmAction === action) {
			clearConfirm();
			if (action === 'update') doApplyUpdate();
			else doRollback();
		} else {
			confirmAction = action;
			if (confirmTimer) clearTimeout(confirmTimer);
			confirmTimer = setTimeout(clearConfirm, 4000);
		}
	}

	function clearConfirm() {
		confirmAction = null;
		if (confirmTimer) { clearTimeout(confirmTimer); confirmTimer = null; }
	}

	async function doApplyUpdate() {
		status = { state: 'running', log: '', reboot_required: false };
		const ok = await withToast(
			() => client.call('system.update.apply'),
			'Update started'
		);
		if (ok !== undefined) {
			startPolling();
		}
	}

	async function doRollback() {
		status = { state: 'running', log: '', reboot_required: false };
		const ok = await withToast(
			() => client.call('system.update.rollback'),
			'Rollback started'
		);
		if (ok !== undefined) {
			startPolling();
		}
	}

	function startPolling() {
		stopPolling();
		pollInterval = setInterval(async () => {
			try {
				status = await client.call<UpdateStatus>('system.update.status');
				if (status && status.state !== 'running') {
					stopPolling();
					await loadVersion();
					if (status.state === 'success') {
						needsRefresh = true;
					}
				}
			} catch {
				// Connection may drop during update, keep polling
			}
		}, 3000);
	}

	function stopPolling() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}
</script>


{#if needsRefresh}
	<div class="mb-4 flex items-center gap-4 rounded-lg border border-blue-800 bg-blue-950 px-4 py-3 text-sm text-blue-200">
		<span class="flex-1">Update applied. Refresh your browser to load the new WebUI.</span>
		<Button variant="secondary" size="xs" onclick={() => location.reload()}>
			Refresh Now
		</Button>
	</div>
{/if}

{#if status?.reboot_required}
	<div class="mb-4 rounded-lg border border-amber-800 bg-amber-950 px-4 py-3 text-sm text-amber-200">
		A kernel update was installed. Use the <strong>Power → Restart</strong> button in the top bar to activate it.
	</div>
{/if}

{#if loading}
	<p class="text-muted-foreground">Loading...</p>
{:else}
	<Card class="mb-6">
		<CardContent class="py-5">
			<!-- Version + status row -->
			<div class="mb-5 flex items-center gap-8">
				<div>
					<div class="mb-0.5 text-xs font-medium uppercase tracking-wide text-muted-foreground">Installed</div>
					<div class="font-mono text-xl font-semibold">{info?.current_version ?? 'unknown'}</div>
				</div>
				{#if info?.latest_version}
					<div class="text-lg text-muted-foreground/30">→</div>
					<div>
						<div class="mb-0.5 text-xs font-medium uppercase tracking-wide text-muted-foreground">Available</div>
						<div class="font-mono text-xl font-semibold {info.update_available ? 'text-blue-400' : ''}">{info.latest_version}</div>
					</div>
				{/if}
				<div class="flex items-end pb-0.5">
					{#if info?.update_available === true}
						<Badge variant="default">Update available</Badge>
					{:else if info?.update_available === false}
						<Badge variant="secondary">Up to date</Badge>
					{/if}
				</div>
			</div>

			<!-- Actions -->
			<div class="flex gap-2">
				<Button size="sm" onclick={checkForUpdates} disabled={checking || status?.state === 'running'}>
					{checking ? 'Checking...' : 'Check for Updates'}
				</Button>
				{#if info?.update_available}
					<Button
						variant={confirmAction === 'update' ? 'destructive' : 'default'}
						size="sm"
						onclick={() => requestAction('update')}
						disabled={status?.state === 'running'}
					>
						{confirmAction === 'update' ? 'Confirm?' : 'Update Now'}
					</Button>
				{/if}
				<Button
					variant={confirmAction === 'rollback' ? 'destructive' : 'secondary'}
					size="sm"
					onclick={() => requestAction('rollback')}
					disabled={status?.state === 'running'}
				>
					{confirmAction === 'rollback' ? 'Confirm?' : 'Rollback'}
				</Button>
			</div>
		</CardContent>
	</Card>

	{#if status && status.state !== 'idle'}
		<Card>
			<CardContent class="py-5">
				<div class="mb-3 flex items-center gap-2">
					<span class="text-sm font-medium text-muted-foreground">Update log</span>
					<Badge variant={
						status.state === 'running' ? 'default' :
						status.state === 'success' ? 'secondary' :
						'destructive'
					}>
						{status.state === 'running' ? 'In progress' :
						 status.state === 'success' ? 'Complete' :
						 'Failed'}
					</Badge>
					{#if status.state === 'running'}
						<span class="inline-block h-2 w-2 animate-pulse rounded-full bg-blue-400"></span>
					{/if}
				</div>
				{#if status.log}
					<pre bind:this={logEl} class="max-h-[32rem] overflow-auto rounded bg-secondary p-3 text-xs leading-relaxed">{status.log}</pre>
				{/if}
			</CardContent>
		</Card>
	{/if}

	<p class="mt-6 text-xs text-muted-foreground">
		Updates are fetched from GitHub and applied using NixOS rebuild.
		The system will atomically switch to the new version, restarting services as needed.
		If anything goes wrong, use Rollback to return to the previous version.
	</p>
{/if}
