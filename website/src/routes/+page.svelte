<script lang="ts">
	import { loadProgress, preloadImages, isLoading } from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / 80);
			return Math.min(frame, 156);
		}
	});

	$effect(() => {
		console.log('Page changed:', scrollY.current);
	});

	onMount(() => {
		preloadImages();
	});
</script>

{#if $isLoading}
	<div class="flex h-screen w-full flex-col items-center justify-center gap-4 bg-black text-white">
		<h1>Loading...</h1>
		<div class="h-8 w-56 border border-dashed border-white">
			<div class="h-full bg-white" style="width: {$loadProgress}%"></div>
		</div>
		<p>{Math.round($loadProgress)}%</p>

		<img src="loading.webp" alt="loading animation" class="absolute right-0 bottom-0 h-32" />
	</div>
{:else}
	<img
		class="sticky top-0 left-0 -z-10 h-screen w-screen object-cover"
		src={`/renders/${current_frame.toString().padStart(4, '0')}.png`}
		alt="scroll animation"
	/>
	<div class="h-3400 -translate-y-[100vh]">
		<div class="flex h-screen w-full flex-col items-center justify-start">
			<div class="flex flex-col items-center gap-0 p-90">
				<h1 class="font-share-tech text-7xl font-bold text-slate-700">Hackxpansion</h1>
				<h2 class="font-share-tech text-2xl font-normal text-slate-500">Make expansion cards!</h2>
			</div>
		</div>
	</div>
{/if}
