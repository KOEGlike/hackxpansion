<script lang="ts">
	import { onMount } from 'svelte';
	import { createWindowCanvasResizer } from '$lib/utils/canvas';

	let canvas = $state<HTMLCanvasElement | undefined>(undefined);
	let canvasResizer: ReturnType<typeof createWindowCanvasResizer> | null = null;

	onMount(() => {
		if (!canvas) {
			console.error('Canvas element not found');
			return;
		}
		const gridCanvas = canvas;
		const parentElement = gridCanvas.parentElement;
		let sizeObserver: ResizeObserver | null = null;

		canvasResizer = createWindowCanvasResizer({
			canvas: gridCanvas,
			onResize: () => drawGrid(gridCanvas)
		});
		canvasResizer.start();

		if (parentElement) {
			sizeObserver = new ResizeObserver(() => {
				canvasResizer?.requestResize();
			});
			sizeObserver.observe(parentElement);
		}

		return () => {
			sizeObserver?.disconnect();
			sizeObserver = null;
			canvasResizer?.stop();
			canvasResizer = null;
		};
	});

	const drawGrid = (canvas: HTMLCanvasElement) => {
		const ctx = canvas.getContext('2d');
		if (!ctx) {
			return;
		}

		const gridSize = 40;
		const width = canvas.width;
		const height = canvas.height;

		ctx.clearRect(0, 0, width, height);
		ctx.strokeStyle = 'rgba(144, 161, 185, 0.2)';
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

<canvas class="pointer-events-none absolute inset-0 z-0 h-full w-full" bind:this={canvas}></canvas>
