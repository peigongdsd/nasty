import { NastyClient } from './rpc';

/** Singleton RPC client shared across all pages */
let instance: NastyClient | null = null;

export function getClient(): NastyClient {
	if (!instance) {
		const wsProto = typeof window !== 'undefined' && window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const host = typeof window !== 'undefined' ? window.location.host : 'localhost';
		instance = new NastyClient(`${wsProto}//${host}/ws`);
	}
	return instance;
}

export function resetClient() {
	if (instance) {
		instance.disconnect();
		instance = null;
	}
}
