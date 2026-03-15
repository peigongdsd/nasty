<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { WebLinksAddon } from '@xterm/addon-web-links';
	import { getToken } from '$lib/auth';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';

	import type { IDisposable } from '@xterm/xterm';

	let terminalEl: HTMLDivElement;
	let term: Terminal | null = null;
	let fitAddon: FitAddon | null = null;
	let ws: WebSocket | null = null;
	let status = $state<'connecting' | 'connected' | 'disconnected'>('connecting');
	let termListeners: IDisposable[] = [];

	onMount(() => {
		term = new Terminal({
			cursorBlink: true,
			fontSize: 14,
			fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
			theme: {
				background: '#0f1117',
				foreground: '#e0e0e0',
				cursor: '#e0e0e0',
				selectionBackground: '#2563eb55',
				black: '#0f1117',
				red: '#dc2626',
				green: '#4ade80',
				yellow: '#f59e0b',
				blue: '#2563eb',
				magenta: '#a855f7',
				cyan: '#22d3ee',
				white: '#e0e0e0',
				brightBlack: '#4b5563',
				brightRed: '#f87171',
				brightGreen: '#86efac',
				brightYellow: '#fcd34d',
				brightBlue: '#60a5fa',
				brightMagenta: '#c084fc',
				brightCyan: '#67e8f9',
				brightWhite: '#ffffff',
			},
		});

		fitAddon = new FitAddon();
		term.loadAddon(fitAddon);
		term.loadAddon(new WebLinksAddon());

		term.open(terminalEl);
		fitAddon.fit();

		connect();

		const resizeObserver = new ResizeObserver(() => {
			fitAddon?.fit();
		});
		resizeObserver.observe(terminalEl);

		return () => {
			resizeObserver.disconnect();
		};
	});

	onDestroy(() => {
		ws?.close();
		term?.dispose();
	});

	function connect() {
		const token = getToken();
		if (!token) {
			status = 'disconnected';
			term?.writeln('\r\n\x1b[31mNot authenticated. Please sign in first.\x1b[0m');
			return;
		}

		const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		ws = new WebSocket(`${proto}//${window.location.host}/ws/terminal`);
		status = 'connecting';

		ws.onopen = () => {
			const cols = term?.cols ?? 80;
			const rows = term?.rows ?? 24;
			ws?.send(JSON.stringify({ token, cols, rows }));
		};

		ws.onmessage = (event) => {
			const data = event.data;

			if (status === 'connecting') {
				try {
					const msg = JSON.parse(data);
					if (msg.authenticated) {
						status = 'connected';

						termListeners.forEach(l => l.dispose());
						termListeners = [];

						termListeners.push(term!.onData((input) => {
							if (ws?.readyState === WebSocket.OPEN) {
								ws.send(input);
							}
						}));

						termListeners.push(term!.onResize(({ cols, rows }) => {
							if (ws?.readyState === WebSocket.OPEN) {
								ws.send(JSON.stringify({ type: 'resize', cols, rows }));
							}
						}));

						return;
					}
					if (msg.error) {
						status = 'disconnected';
						term?.writeln(`\r\n\x1b[31m${msg.error}\x1b[0m`);
						return;
					}
				} catch {
					// Not JSON, treat as terminal output
				}
			}

			term?.write(data);
		};

		ws.onclose = () => {
			if (status === 'connected') {
				term?.writeln('\r\n\x1b[33mConnection closed.\x1b[0m');
			}
			status = 'disconnected';
		};

		ws.onerror = () => {
			status = 'disconnected';
		};
	}

	function reconnect() {
		ws?.close();
		term?.clear();
		connect();
	}
</script>

<div class="flex h-[calc(100vh-4rem)] flex-col gap-4">
	<div class="flex shrink-0 items-center justify-between">
		<h2 class="text-2xl font-bold">Terminal</h2>
		<div class="flex items-center gap-3">
			<span class="text-xs uppercase {
				status === 'connected' ? 'text-green-400' :
				status === 'connecting' ? 'text-amber-500' : 'text-muted-foreground'
			}">{status}</span>
			{#if status === 'disconnected'}
				<Button size="sm" onclick={reconnect}>Reconnect</Button>
			{/if}
		</div>
	</div>

	<!-- bcachefs cheatsheet -->
	<details class="shrink-0 rounded-lg border border-border bg-card text-xs">
		<summary class="cursor-pointer select-none px-4 py-2 font-medium text-muted-foreground hover:text-foreground">
			bcachefs commands
		</summary>
		<div class="grid grid-cols-1 gap-x-8 gap-y-1 px-4 pb-3 pt-2 sm:grid-cols-2 font-mono">
			{#each [
				['bcachefs fs usage /storage/<pool>',     'Space by type (superblock, journal, btree, data, cached, parity)'],
				['bcachefs fs top /storage/<pool>',       'Live btree ops per process — interactive, q to quit'],
				['bcachefs fs timestats /storage/<pool>', 'Live op latency (min/max/mean/stddev/EWMA) — interactive, q to quit'],
				['bcachefs show-super /dev/<disk>',       'Dump filesystem superblock (UUID, features, device list)'],
				['bcachefs fs usage -h /storage/<pool>',  'Human-readable sizes'],
				['bcachefs device list /storage/<pool>',  'Show all member devices with state and tier'],
				['bcachefs device add /storage/<pool> /dev/<disk>', 'Add a device to an existing pool'],
				['bcachefs device remove /storage/<pool> /dev/<disk>', 'Remove a device (triggers rebalance)'],
				['bcachefs device set-state failed /dev/<disk>', 'Mark a device failed'],
				['bcachefs data rereplicate /storage/<pool>', 'Rereplicate data after adding a device or device failure'],
				['bcachefs subvolume list /storage/<pool>', 'List all subvolumes'],
				['bcachefs subvolume snapshot <src> <dst>', 'Create a snapshot'],
			] as [cmd, desc]}
				<code class="text-cyan-400">{cmd}</code>
				<span class="text-muted-foreground">{desc}</span>
			{/each}
		</div>
	</details>

	<div class="flex-1 overflow-hidden rounded-lg border border-border p-1" style="background: #0f1117" bind:this={terminalEl}></div>
</div>

<style>
	@import '@xterm/xterm/css/xterm.css';

	div :global(.xterm) {
		height: 100%;
	}

	div :global(.xterm-viewport) {
		overflow-y: auto !important;
	}
</style>
