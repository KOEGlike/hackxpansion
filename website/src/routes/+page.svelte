<script lang="ts">
	import { loadProgress, preloadImages, isLoading } from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';
	import { fade } from 'svelte/transition';

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / 80);
			return Math.min(frame, 156);
		}
	});

	$effect(() => {
		console.log('frame:', current_frame);
	});

	onMount(() => {
		preloadImages();
	});

	const frame_events = [
		{ start: 0, end: 9 },
		{ start: 16, end: 36 },
		{ start: 54, end: 79 },
		{ start: 93, end: 114 },
		{ start: 130, end: 999 },
		{ start: 155, end: 999 }
	];
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
	<div class="sticky top-0 left-0 h-screen w-screen">
		<img
			class="-z-10 h-screen w-screen object-cover"
			src={`/renders/${current_frame.toString().padStart(4, '0')}.png`}
			alt="scroll animation"
		/>
		<div class="h-screen w-screen -translate-y-[100vh]">
			<!-- Title -->
			{#if current_frame < frame_events[0].end && current_frame >= frame_events[0].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 p-90">
						<h1 class="font-share-tech text-7xl font-bold text-slate-700">Hackxpansion</h1>
						<h2 class="font-share-tech text-2xl font-normal text-slate-500">
							Make expansion cards!
						</h2>
					</div>
				</div>
			{/if}
			<!-- Step 1 -->
			{#if current_frame < frame_events[1].end && current_frame >= frame_events[1].start}
				<div
					class="absolute right-20 bottom-70 flex flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0">
						<h1 class="font-share-tech text-7xl font-bold text-slate-700">Step 1</h1>
						<h2 class="font-share-tech text-2xl font-normal text-slate-500">Design your card!</h2>
					</div>
				</div>
			{/if}
			<!-- Step 2 -->
			{#if current_frame < frame_events[2].end && current_frame >= frame_events[2].start}
				<div
					class="flex h-full w-full items-center justify-center gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-row items-center justify-center gap-10">
						<img class="w-64" src="/ferris.png" alt="ferris" />
						<div class="flex flex-col items-center">
							<h1 class="font-share-tech text-7xl font-bold text-slate-700">Step 2</h1>
							<h2 class="font-share-tech text-2xl font-normal text-slate-500">Code a driver!</h2>
						</div>
					</div>
				</div>
			{/if}
			<!-- Step 3 -->
			{#if current_frame < frame_events[3].end && current_frame >= frame_events[3].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 p-50">
						<h1 class="font-share-tech text-7xl font-bold text-slate-700">Step 3</h1>
						<h2 class="font-share-tech text-2xl font-normal text-slate-500">Make 3 more!</h2>
					</div>
				</div>
			{/if}
			<!-- Step 4 -->
			{#if current_frame < frame_events[4].end && current_frame >= frame_events[4].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 p-30">
						<h1 class="font-share-tech text-7xl font-bold text-slate-700">Step 4</h1>
						{#if current_frame < frame_events[5].end && current_frame >= frame_events[5].start}
							<h2 class="font-share-tech text-2xl font-normal text-slate-500">
								Submit and get the console!
							</h2>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
	<div class="h-3400 -translate-y-[100vh]"></div>
{/if}
