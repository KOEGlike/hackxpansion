<script lang="ts">
	import { loadProgress, preloadImages, isLoading, imageCount } from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';
	import { fade } from 'svelte/transition';
	import { asset, resolve } from '$app/paths';
	import Scroll from '$lib/components/scroll.svelte';
	import DocsButton from '$lib/components/docs_button.svelte';

	const scrollPerFrame = 25;

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / scrollPerFrame);
			return Math.min(frame, imageCount - 1);
		}
	});

	$effect(() => {
		console.log('frame:', current_frame);
	});

	onMount(() => {
		preloadImages();
	});

	const frame_events = [
		{ start: 0, end: 12 },
		{ start: 53, end: 97 },
		{ start: 126, end: 193 },
		{ start: 236, end: 281 },
		{ start: 326, end: 999 },
		{ start: imageCount - 2, end: 999 }
	];

	const isCurrentFrame = (index: number) => {
		return current_frame >= frame_events[index].start && current_frame < frame_events[index].end;
	};
</script>

{#if $isLoading}
	<div class="flex h-screen w-full flex-col items-center justify-center gap-4 bg-black text-white">
		<h1>Loading...</h1>
		<div class="h-8 w-56 border border-dashed border-white">
			<div class="h-full bg-white" style="width: {$loadProgress}%"></div>
		</div>
		<p>{Math.round($loadProgress)}%</p>
		<a href={resolve('/simple')} class="hover:underline">Not loading? Click me for simple version</a
		>
		<img src="loading.webp" alt="loading animation" class="absolute right-0 bottom-0 h-32" />
	</div>
{:else}
	<div
		class="overflow-y-scroll"
		style="height: calc({imageCount * scrollPerFrame}px + 100vh)"
	></div>
	<div class="fixed top-0 left-0 h-screen w-screen">
		<enhanced:img
			class="-z-10 h-screen w-screen object-cover"
			src={`${asset(`/renders/${current_frame.toString().padStart(4, '0')}.webp`)}`}
			alt="scroll animation"
		/>
		<div class="h-screen w-screen -translate-y-[100vh]">
			<DocsButton />
			<!-- Title -->
			{#if isCurrentFrame(0)}
				<div
					class="flex h-full w-full flex-col items-center justify-between gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-80 sm:pt-90">
						<h1 class="text-5xl font-bold text-slate-700 sm:text-7xl">Hackxpansion</h1>
						<h2 class="w-80 text-center text-xl font-normal text-slate-500 sm:w-100 sm:text-2xl">
							Make expansion cards, get a custom console to use them in!
						</h2>
					</div>
					<div class="mb-20 flex flex-col items-center justify-center">
						<a
							href="https://meko.fillout.com/hackxpansion"
							target="_blank"
							rel="noopener noreferrer"
							class="w-fitindent-4 pb-5 text-xl text-slate-700 hover:underline">RSVP</a
						>
						<Scroll extraclass="h-11 w-fit fill-slate-700" />
					</div>
				</div>
			{/if}

			<!-- Step 1 -->
			{#if isCurrentFrame(1)}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-40 sm:pt-50">
						<h1 class="  text-6xl font-bold text-slate-700 sm:text-7xl">Step 1</h1>
						<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">Design your card!</h2>
					</div>
				</div>
			{/if}

			<!-- Step 2 -->
			{#if isCurrentFrame(2)}
				<div
					class="flex h-full w-full items-center justify-center gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center justify-center gap-10 sm:flex-row">
						<img class="w-64" src={`${asset('/ferris.webp')}`} alt="ferris" />
						<div class="flex flex-col items-center">
							<h1 class="text-6xl font-bold text-slate-700 sm:text-7xl">Step 2</h1>
							<h2 class="text-xl font-normal text-slate-500 sm:text-2xl">Code a driver!</h2>
						</div>
					</div>
				</div>
			{/if}

			<!-- Step 3 -->
			{#if isCurrentFrame(3)}
				<div
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-40 sm:pt-50">
						<h1 class=" text-6xl font-bold text-slate-700 sm:text-7xl">Step 3</h1>
						<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">Make 3 more!</h2>
					</div>
				</div>
			{/if}

			<!-- Step 4 -->
			{#if isCurrentFrame(4)}
				<div
					class="flex h-full w-full flex-col items-center justify-between gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 pt-15">
						<h1 class="text-6xl font-bold text-slate-700 sm:text-7xl">Step 4</h1>
						{#if isCurrentFrame(5)}
							<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">
								Submit and get the console!
							</h2>
						{/if}
					</div>
					{#if isCurrentFrame(5)}
						<button
							class="mb-20 content-box p-3 text-3xl text-slate-700 hover:content-box-hover sm:mb-40"
							onclick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
						>
							Go Up
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	:global(html) {
		scroll-snap-type: y proximity;
	}
</style>
