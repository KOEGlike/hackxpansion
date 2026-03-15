<script lang="ts">
	import { loadProgress, preloadAssets, isLoading, frameCount } from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';
	import { fade } from 'svelte/transition';
	import { asset } from '$app/paths';

	let heroVideo = $state<HTMLVideoElement | null>(null);
	let videoDuration = $state(0);
	let seekInProgress = $state(false);
	let pendingSeekTime = $state<number | null>(null);

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / 32);
			return Math.max(0, Math.min(frame, frameCount - 1));
		}
	});

	function flushVideoSeek() {
		if (!heroVideo || pendingSeekTime === null) {
			return;
		}

		const nextTime = pendingSeekTime;
		pendingSeekTime = null;
		seekInProgress = true;

		try {
			if (typeof heroVideo.fastSeek === 'function') {
				heroVideo.fastSeek(nextTime);
			} else {
				heroVideo.currentTime = nextTime;
			}
		} catch {
			heroVideo.currentTime = nextTime;
		}
	}

	function queueVideoSeek(targetTime: number) {
		if (!heroVideo || videoDuration <= 0) {
			return;
		}

		const clampedTime = Math.max(0, Math.min(targetTime, videoDuration));
		pendingSeekTime = clampedTime;

		if (!seekInProgress) {
			flushVideoSeek();
		}
	}

	function handleVideoSeeked() {
		seekInProgress = false;

		if (pendingSeekTime !== null) {
			flushVideoSeek();
		}
	}

	$effect(() => {
		if (!heroVideo || videoDuration <= 0) {
			return;
		}

		const progress = current_frame / Math.max(frameCount - 1, 1);
		const targetTime = progress * videoDuration;

		if (Math.abs(heroVideo.currentTime - targetTime) > 0.05) {
			queueVideoSeek(targetTime);
		}
	});

	onMount(() => {
		preloadAssets();
	});

	const frame_events = [
		{ start: 0, end: 22 },
		{ start: 40, end: 90 },
		{ start: 135, end: 197 },
		{ start: 232, end: 285 },
		{ start: 325, end: frameCount },
		{ start: 387, end: frameCount }
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
		<video
			bind:this={heroVideo}
			class="-z-10 h-screen w-screen object-cover"
			onloadedmetadata={() => {
				if (heroVideo) {
					videoDuration = heroVideo.duration;
					queueVideoSeek(0);
				}
			}}
			onseeked={handleVideoSeeked}
			muted
			playsinline
			preload="auto"
		>
			<source src={asset('/renders/output_h264.mp4')} type="video/mp4" />
			<source src={asset('/renders/output_vp9.webm')} type="video/webm" />
			<source src={asset('/renders/output_av1.webm')} type="video/webm; codecs=av01.0.08M.08" />
		</video>
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
					class="flex h-full w-full flex-col items-center justify-start gap-0"
					transition:fade={{ duration: 100 }}
				>
					<div class="flex flex-col items-center gap-0 p-50">
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
						<img class="w-64" src={`${asset('/ferris.png')}`} alt="ferris" />
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
