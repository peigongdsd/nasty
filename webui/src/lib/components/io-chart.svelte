<script lang="ts">
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { AreaChart } from 'layerchart';
	import { scaleUtc } from 'd3-scale';
	import { curveMonotoneX } from 'd3-shape';
	import { formatBytes, makeBytesFormatter } from '$lib/format';

	interface Props {
		samples: { time: Date; in: number; out: number }[];
		inLabel: string;
		outLabel?: string;
		inColor?: string;
		outColor?: string;
		yFormat?: (v: number) => string;
		tooltipFormat?: (v: number) => string;
	}

	let {
		samples,
		inLabel,
		outLabel,
		inColor = 'var(--chart-1)',
		outColor = 'var(--chart-2)',
		yFormat,
		tooltipFormat = (v: number) => formatBytes(v) + '/s',
	}: Props = $props();

	// Derive the Y-axis formatter from the peak value so all ticks share one unit.
	const yFormatDerived = $derived.by(() => {
		if (yFormat) return yFormat;
		const max = Math.max(0, ...samples.map((s) => Math.max(s.in, s.out)));
		return (v: number) => makeBytesFormatter(max)(v) + '/s';
	});

	const singleSeries = $derived(!outLabel);

	const chartConfig = $derived.by((): Chart.ChartConfig => {
		const cfg: Chart.ChartConfig = { in: { label: inLabel, color: inColor } };
		if (!singleSeries) cfg.out = { label: outLabel!, color: outColor };
		return cfg;
	});

	const series = $derived(
		singleSeries
			? [{ key: 'in', label: inLabel, color: inColor }]
			: [
					{ key: 'in', label: inLabel, color: inColor },
					{ key: 'out', label: outLabel!, color: outColor },
				]
	);
</script>

{#if samples.length >= 2}
	<Chart.Container config={chartConfig} class="aspect-[4/1] w-full pl-20">
		<AreaChart
			data={samples}
			x="time"
			xScale={scaleUtc()}
			{series}
			props={{
				area: {
					curve: curveMonotoneX,
					'fill-opacity': 0.3,
					line: { class: 'stroke-1' },
				},
				xAxis: {
					format: (v: Date) => v.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
					ticks: 4,
				},
				yAxis: {
					format: yFormatDerived,
					ticks: 3,
				},
			}}
		>
			{#snippet tooltip()}
				<Chart.Tooltip
					labelFormatter={(v: Date) => v.toLocaleTimeString()}
					indicator="line"
				>
					{#snippet formatter({ value })}
						<span class="text-foreground font-mono font-medium tabular-nums">{tooltipFormat(value as number)}</span>
					{/snippet}
				</Chart.Tooltip>
			{/snippet}
		</AreaChart>
	</Chart.Container>
{:else}
	<div class="flex h-16 items-center justify-center text-xs text-muted-foreground">
		Collecting data...
	</div>
{/if}
