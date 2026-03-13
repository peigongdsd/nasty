/**
 * Rolling history buffer for dashboard time-series charts.
 * Stores per-resource rates (bytes/s).
 * For the 5m range this acts as a rolling 60-sample buffer;
 * for longer ranges it holds the full bucketed history loaded from the server.
 */

const MAX_SAMPLES = 400; // enough for any bucketed range (max ~360 points)

export interface IoRateHistory {
	/** Per-resource history: resource name -> array of {time, in, out} */
	resources: Map<string, { time: Date; in: number; out: number }[]>;
}

function createHistory(): IoRateHistory {
	return { resources: new Map() };
}

function pushSample(
	history: IoRateHistory,
	name: string,
	time: Date,
	inRate: number,
	outRate: number,
) {
	let samples = history.resources.get(name);
	if (!samples) {
		samples = [];
		history.resources.set(name, samples);
	}
	samples.push({ time, in: inRate, out: outRate });
	if (samples.length > MAX_SAMPLES) {
		samples.splice(0, samples.length - MAX_SAMPLES);
	}
}

/**
 * Reactive history store using Svelte 5 runes.
 */
export function createIoHistory() {
	let history = $state<IoRateHistory>(createHistory());

	return {
		get resources() {
			return history.resources;
		},
		push(name: string, time: Date, inRate: number, outRate: number) {
			pushSample(history, name, time, inRate, outRate);
			// Trigger reactivity by reassigning
			history = { resources: new Map(history.resources) };
		},
		getSamples(name: string) {
			return history.resources.get(name) ?? [];
		},
		clear() {
			history = createHistory();
		},
	};
}
