<script lang="ts">
	import { loadProgress, preloadImages, isLoading } from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';
	import { fade } from 'svelte/transition';
	import { asset } from '$app/paths';

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / 80);
			return Math.min(frame, 156);
		}
	});

	// let current_frame = 60;

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
	<div class="h-3400 overflow-y-scroll"></div>
	<div class="fixed top-0 left-0 h-screen w-screen">
		<enhanced:img
			class="-z-10 h-screen w-screen object-cover"
			src={`${asset(`/renders/${current_frame.toString().padStart(4, '0')}.webp`)}`}
			alt="scroll animation"
		/>
		<div class="h-screen w-screen -translate-y-[100vh]">
			<!-- Title -->
			{#if current_frame < frame_events[0].end && current_frame >= frame_events[0].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-75 sm:pt-90">
						<h1 class="font-share-tech text-5xl font-bold text-slate-700 sm:text-7xl">
							Hackxpansion
						</h1>
						<h2 class="font-share-tech text-xl font-normal text-slate-500 sm:text-2xl">
							Make expansion cards!
						</h2>
					</div>
				</div>
			{/if}
			<!-- Step 1 -->
			{#if current_frame < frame_events[1].end && current_frame >= frame_events[1].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-40 sm:pt-50">
						<h1 class=" font-share-tech text-6xl font-bold text-slate-700 sm:text-7xl">Step 1</h1>
						<h2 class="font-share-tech text-xl font-normal text-slate-500 sm:text-2xl">
							Design your card!
						</h2>
					</div>
				</div>
			{/if}
			<!-- Step 2 -->
			{#if current_frame < frame_events[2].end && current_frame >= frame_events[2].start}
				<div
					class="flex h-full w-full items-center justify-center gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center justify-center gap-10 sm:flex-row">
						<img class="w-64" src={`${asset('/ferris.png')}`} alt="ferris" />
						<div class="flex flex-col items-center">
							<h1 class="font-share-tech text-6xl font-bold text-slate-700 sm:text-7xl">Step 2</h1>
							<h2 class="font-share-tech text-xl font-normal text-slate-500 sm:text-2xl">
								Code a driver!
							</h2>
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
					<div class="flex flex-col items-center gap-0 pt-40 sm:pt-50">
						<h1 class="font-share-tech text-6xl font-bold text-slate-700 sm:text-7xl">Step 3</h1>
						<h2 class="font-share-tech text-xl font-normal text-slate-500 sm:text-2xl">
							Make 3 more!
						</h2>
					</div>
				</div>
			{/if}
			<!-- Step 4 -->
			{#if current_frame < frame_events[4].end && current_frame >= frame_events[4].start}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-15">
						<h1 class="font-share-tech text-6xl font-bold text-slate-700 sm:text-7xl">Step 4</h1>
						{#if current_frame < frame_events[5].end && current_frame >= frame_events[5].start}
							<h2 class="font-share-tech text-xl font-normal text-slate-500 sm:text-2xl">
								Submit and get the console!
							</h2>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	:global(body) {
		@apply h-screen overflow-x-hidden bg-[#c3cbd4];
	}
</style>
