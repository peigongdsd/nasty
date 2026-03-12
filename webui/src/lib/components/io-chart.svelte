<script lang="ts">
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { AreaChart } from 'layerchart';
	import { scaleUtc } from 'd3-scale';
	import { curveMonotoneX } from 'd3-shape';
	import { formatBytes } from '$lib/format';

	interface Props {
		samples: { time: Date; in: number; out: number }[];
		inLabel: string;
		outLabel: string;
		inColor?: string;
		outColor?: string;
	}

	let {
		samples,
		inLabel,
		outLabel,
		inColor = 'var(--chart-1)',
		outColor = 'var(--chart-2)',
	}: Props = $props();

	const chartConfig = $derived({
		in: { label: inLabel, color: inColor },
		out: { label: outLabel, color: outColor },
	} satisfies Chart.ChartConfig);
</script>

{#if samples.length >= 2}
	<Chart.Container config={chartConfig} class="aspect-[4/1] w-full">
		<AreaChart
			data={samples}
			x="time"
			xScale={scaleUtc()}
			series={[
				{ key: 'in', label: inLabel, color: inColor },
				{ key: 'out', label: outLabel, color: outColor },
			]}
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
					format: (v: number) => formatBytes(v) + '/s',
					ticks: 3,
				},
			}}
		>
			{#snippet tooltip()}
				<Chart.Tooltip
					labelFormatter={(v: Date) => v.toLocaleTimeString()}
					valueFormatter={(v: number) => formatBytes(v) + '/s'}
					indicator="line"
				/>
			{/snippet}
		</AreaChart>
	</Chart.Container>
{:else}
	<div class="flex h-16 items-center justify-center text-xs text-muted-foreground">
		Collecting data...
	</div>
{/if}
