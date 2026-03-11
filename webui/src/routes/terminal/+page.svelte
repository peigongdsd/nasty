<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { WebLinksAddon } from '@xterm/addon-web-links';
	import { getToken } from '$lib/auth';

	let terminalEl: HTMLDivElement;
	let term: Terminal | null = null;
	let fitAddon: FitAddon | null = null;
	let ws: WebSocket | null = null;
	let status = $state<'connecting' | 'connected' | 'disconnected'>('connecting');

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

			// Check for JSON control messages
			if (status === 'connecting') {
				try {
					const msg = JSON.parse(data);
					if (msg.authenticated) {
						status = 'connected';

						// Wire up input: terminal → WebSocket
						term?.onData((input) => {
							if (ws?.readyState === WebSocket.OPEN) {
								ws.send(input);
							}
						});

						// Wire up resize
						term?.onResize(({ cols, rows }) => {
							if (ws?.readyState === WebSocket.OPEN) {
								ws.send(JSON.stringify({ type: 'resize', cols, rows }));
							}
						});

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

			// Regular terminal output
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

<div class="terminal-page">
	<div class="terminal-header">
		<h2>Terminal</h2>
		<div class="terminal-controls">
			<span class="status" class:connected={status === 'connected'} class:connecting={status === 'connecting'}>
				{status}
			</span>
			{#if status === 'disconnected'}
				<button onclick={reconnect}>Reconnect</button>
			{/if}
		</div>
	</div>
	<div class="terminal-container" bind:this={terminalEl}></div>
</div>

<style>
	@import '@xterm/xterm/css/xterm.css';

	.terminal-page {
		display: flex;
		flex-direction: column;
		height: calc(100vh - 4rem);
	}

	.terminal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
		flex-shrink: 0;
	}

	.terminal-header h2 {
		margin: 0;
		font-size: 1.5rem;
	}

	.terminal-controls {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.status {
		font-size: 0.75rem;
		text-transform: uppercase;
		color: #6b7280;
	}

	.status.connected {
		color: #4ade80;
	}

	.status.connecting {
		color: #f59e0b;
	}

	.terminal-container {
		flex: 1;
		border: 1px solid #2d3348;
		border-radius: 8px;
		overflow: hidden;
		padding: 4px;
		background: #0f1117;
	}

	.terminal-container :global(.xterm) {
		height: 100%;
	}

	.terminal-container :global(.xterm-viewport) {
		overflow-y: auto !important;
	}
</style>
