/** Shared reactive terminal connection status for the top bar indicator. */
let _status = $state<'connecting' | 'connected' | 'disconnected' | 'idle'>('idle');

export const terminalStatus = {
	get value() { return _status; },
	set(s: 'connecting' | 'connected' | 'disconnected' | 'idle') { _status = s; },
};
