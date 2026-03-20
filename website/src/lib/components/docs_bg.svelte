<script lang="ts">
	import { onMount } from 'svelte';
	import { createWindowCanvasResizer } from '$lib/utils/canvas';

	let canvas = $state<HTMLCanvasElement | null>(null);
	let canvasResizer: ReturnType<typeof createWindowCanvasResizer> | null = null;

	onMount(() => {
		canvasResizer = createWindowCanvasResizer({ getCanvas: () => canvas });
		canvasResizer.start();

		return () => {
			canvasResizer?.stop();
			canvasResizer = null;
		};
	});
</script>

<canvas bind:this={canvas}></canvas>
