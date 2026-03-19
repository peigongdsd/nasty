<script lang="ts">
	import { onMount } from 'svelte';

	// Each ring: size, colors, spin duration, direction, dot size
	const rings = [
		{ size: 208, color: '#eab308', head: '#fde047', glow: '#eab30880', dur: 2.5, dir: 1, dot: 12, trail: 3 },
		{ size: 176, color: '#f97316', head: '#fdba74', glow: '#f9731680', dur: 3,   dir: -1, dot: 10, trail: 2.5 },
		{ size: 144, color: '#ef4444', head: '#fca5a5', glow: '#ef444480', dur: 3.5, dir: 1, dot: 10, trail: 2.5 },
		{ size: 112, color: '#22c55e', head: '#86efac', glow: '#22c55e80', dur: 4,   dir: -1, dot: 8, trail: 2 },
	];

	// Random spawn positions for each ring (angle + distance from center)
	const spawns = rings.map(() => ({
		angle: Math.random() * 360,
		dist: 250 + Math.random() * 150, // spawn 250-400px out
	}));

	let entered = $state(false);
	let spinning = $state(false);

	onMount(() => {
		// Small delay before entry animation starts
		const t1 = setTimeout(() => { entered = true; }, 50);
		// After entry animation completes, enable spinning
		const t2 = setTimeout(() => { spinning = true; }, 1100);
		return () => { clearTimeout(t1); clearTimeout(t2); };
	});
</script>

<div class="relative flex items-center justify-center" style="width: 220px; height: 220px;">
	{#each rings as ring, i}
		{@const spawn = spawns[i]}
		{@const startX = Math.cos((spawn.angle * Math.PI) / 180) * spawn.dist}
		{@const startY = Math.sin((spawn.angle * Math.PI) / 180) * spawn.dist}
		{@const delay = i * 80}

		<!-- Comet trail -->
		<div
			class="absolute rounded-full"
			class:animate-none={!spinning}
			style="
				width: {ring.size}px; height: {ring.size}px;
				background: conic-gradient(from 0deg, transparent 40%, {ring.color}40 70%, {ring.color} 90%, {ring.head} 100%);
				-webkit-mask: radial-gradient(farthest-side, transparent calc(100% - {ring.trail}px), #000 calc(100% - {ring.trail}px));
				mask: radial-gradient(farthest-side, transparent calc(100% - {ring.trail}px), #000 calc(100% - {ring.trail}px));
				transform: translate({entered ? 0 : startX}px, {entered ? 0 : startY}px) scale({entered ? 1 : 0.3});
				opacity: {entered ? 1 : 0};
				transition: transform 1s cubic-bezier(0.22, 1, 0.36, 1) {delay}ms, opacity 0.6s ease {delay}ms;
				{spinning ? `animation: spin ${ring.dur}s linear infinite ${ring.dir < 0 ? 'reverse' : ''};` : ''}
			"
		></div>

		<!-- Comet head dot -->
		<div
			class="absolute"
			class:animate-none={!spinning}
			style="
				width: {ring.size}px; height: {ring.size}px;
				transform: translate({entered ? 0 : startX}px, {entered ? 0 : startY}px) scale({entered ? 1 : 0.3});
				opacity: {entered ? 1 : 0};
				transition: transform 1s cubic-bezier(0.22, 1, 0.36, 1) {delay}ms, opacity 0.6s ease {delay}ms;
				{spinning ? `animation: spin ${ring.dur}s linear infinite ${ring.dir < 0 ? 'reverse' : ''};` : ''}
			"
		>
			<div
				class="absolute left-1/2 rounded-full"
				style="
					top: -{ring.dot / 2 / 4}rem;
					width: {ring.dot}px; height: {ring.dot}px;
					transform: translateX(-50%);
					background: {ring.head};
					box-shadow: 0 0 10px {ring.head}, 0 0 24px {ring.color}, 0 0 44px {ring.glow};
				"
			></div>
		</div>

		<!-- Static base ring -->
		<div
			class="absolute rounded-full"
			style="
				width: {ring.size}px; height: {ring.size}px;
				border: 1px solid {ring.color}18;
				opacity: {entered ? 1 : 0};
				transition: opacity 0.8s ease {delay + 300}ms;
			"
		></div>
	{/each}

	<span
		class="text-sm text-muted-foreground"
		style="opacity: {entered ? 1 : 0}; transition: opacity 0.8s ease 400ms;"
	>Reconnecting...</span>
</div>
