<script lang="ts">
	// Each ring starts spinning from a random angle so they don't all begin aligned.
	const rings = [
		{ size: 208, color: '#eab308', head: '#fde047', glow: '#eab30880', dur: 2.5, dir: '', dot: 12, trail: 3, border: '#eab30818', startDeg: Math.floor(Math.random() * 360) },
		{ size: 176, color: '#f97316', head: '#fdba74', glow: '#f9731680', dur: 3,   dir: 'reverse', dot: 10, trail: 2.5, border: '#f9731618', startDeg: Math.floor(Math.random() * 360) },
		{ size: 144, color: '#ef4444', head: '#fca5a5', glow: '#ef444480', dur: 3.5, dir: '', dot: 10, trail: 2.5, border: '#ef444418', startDeg: Math.floor(Math.random() * 360) },
		{ size: 112, color: '#22c55e', head: '#86efac', glow: '#22c55e80', dur: 4,   dir: 'reverse', dot: 8, trail: 2, border: '#22c55e18', startDeg: Math.floor(Math.random() * 360) },
	];
</script>

<div class="relative flex items-center justify-center" style="width: 220px; height: 220px;">
	{#each rings as ring}
		{@const r = ring.size / 2}
		{@const dotR = ring.dot / 2}
		<!-- Head angle: conic-gradient "from Xdeg" means 100% lands at Xdeg (0deg = top, clockwise) -->
		{@const headAngle = ring.startDeg}
		{@const headRad = (headAngle - 90) * Math.PI / 180}
		{@const dotX = r + r * Math.cos(headRad) - dotR}
		{@const dotY = r + r * Math.sin(headRad) - dotR}

		<!-- Comet trail + head as one spinning unit -->
		<div
			class="absolute"
			style="
				width: {ring.size}px; height: {ring.size}px;
				animation: spin {ring.dur}s linear infinite {ring.dir};
			"
		>
			<!-- Trail — flip gradient for clockwise rings so the bright head leads -->
			<div
				class="absolute inset-0 rounded-full"
				style="
					background: conic-gradient(from {ring.startDeg}deg, {ring.head} 0%, {ring.color} 10%, {ring.color}40 30%, transparent 60%);
					-webkit-mask: radial-gradient(farthest-side, transparent calc(100% - {ring.trail}px), #000 calc(100% - {ring.trail}px));
					mask: radial-gradient(farthest-side, transparent calc(100% - {ring.trail}px), #000 calc(100% - {ring.trail}px));
				"
			></div>
			<!-- Head dot — positioned at the gradient's 100% point -->
			<div
				class="absolute rounded-full"
				style="
					left: {dotX}px; top: {dotY}px;
					width: {ring.dot}px; height: {ring.dot}px;
					background: {ring.head};
					box-shadow: 0 0 10px {ring.head}, 0 0 24px {ring.color}, 0 0 44px {ring.glow};
				"
			></div>
		</div>

		<!-- Static base ring -->
		<div class="absolute rounded-full" style="width: {ring.size}px; height: {ring.size}px; border: 1px solid {ring.border};"></div>
	{/each}
	<span class="text-sm text-muted-foreground">Reconnecting...</span>
</div>
