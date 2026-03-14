<script lang="ts">
	import { loadProgress, preloadImages, isLoading } from '$lib/stores/loading';
	import { onMount } from 'svelte';

	onMount(() => {
		preloadImages();
	});
</script>

{#if $isLoading}
	<div class="flex h-screen w-full flex-col items-center justify-center gap-4">
		<h1>Loading...</h1>
		<div class="progress-bar">
			<div class="progress-fill" style="width: {$loadProgress}%"></div>
		</div>
		<p>{Math.round($loadProgress)}%</p>
	</div>
{/if}
