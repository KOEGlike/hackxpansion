<script lang="ts">
	import { onMount } from 'svelte';
	import { createWindowCanvasResizer } from '$lib/utils/canvas';

	let canvas = $state<HTMLCanvasElement | undefined>(undefined);
	let canvasResizer: ReturnType<typeof createWindowCanvasResizer> | null = null;

	onMount(() => {
		if (!canvas) {
			return;
		}

		canvasResizer = createWindowCanvasResizer({ canvas });
		canvasResizer.start();

		drawGrid(canvas);

		return () => {
			canvasResizer?.stop();
			canvasResizer = null;
		};
	});

	const drawGrid = (canvas: HTMLCanvasElement) => {
		const ctx = canvas.getContext('2d');
		if (!ctx) {
			return;
		}

		const gridSize = 50;
		const width = canvas.width;
		const height = canvas.height;

		ctx.clearRect(0, 0, width, height);
		ctx.strokeStyle = 'rgba(0, 0, 0, 0.1)';
		ctx.lineWidth = 1;

		for (let x = 0; x < width; x += gridSize) {
			ctx.beginPath();
			ctx.moveTo(x, 0);
			ctx.lineTo(x, height);
			ctx.stroke();
		}

		for (let y = 0; y < height; y += gridSize) {
			ctx.beginPath();
			ctx.moveTo(0, y);
			ctx.lineTo(width, y);
			ctx.stroke();
		}
	};
</script>

<canvas class="fixed inset-0 h-screen w-full" bind:this={canvas}></canvas>
