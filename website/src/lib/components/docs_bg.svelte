<script lang="ts">
	import { onMount } from 'svelte';
	import { createWindowCanvasResizer } from '$lib/utils/canvas';

	let canvas = $state<HTMLCanvasElement | undefined>(undefined);
	let canvasResizer: ReturnType<typeof createWindowCanvasResizer> | null = null;

	onMount(() => {
		if (canvas) {
			canvasResizer = createWindowCanvasResizer({ canvas });
			canvasResizer.start();
		}

		return () => {
			canvasResizer?.stop();
			canvasResizer = null;
		};
	});
</script>

<canvas bind:this={canvas}></canvas>
