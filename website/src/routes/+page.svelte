<script lang="ts">
	import {
		loadProgress,
		preloadImages,
		isLoading,
		imageCount,
		preloadedFrames
	} from '$lib/stores/loading';
	import { onMount } from 'svelte';
	import { scrollY } from 'svelte/reactivity/window';
	import { fade } from 'svelte/transition';
	import { asset, resolve } from '$app/paths';
	import { createWindowCanvasResizer } from '$lib/utils/canvas';
	import LandingSection from '$lib/components/landing_section.svelte';
	import TopBar from '$lib/components/top_bar.svelte';
	import StepHeading from '$lib/components/step_heading.svelte';
	import { landingContent } from '$lib/content/content';
	import Footer from '$lib/components/footer.svelte';
	import Faq from '$lib/components/faq.svelte';

	const scrollPerFrame = 25;
	let frameCanvas = $state<HTMLCanvasElement | undefined>(undefined);
	let frameCanvasResizer: ReturnType<typeof createWindowCanvasResizer> | null = null;

	let current_frame = $derived.by(() => {
		if (!scrollY.current) {
			return 0;
		} else {
			const frame = Math.floor(scrollY.current / scrollPerFrame);
			return Math.min(frame, imageCount - 1);
		}
	});

	let currentFrameImage = $derived($preloadedFrames[current_frame]);
	const animationScrollDistance = (imageCount - 1) * scrollPerFrame;
	let scrollProgress = $derived.by(() => {
		if (!scrollY.current || imageCount <= 1) {
			return 0;
		}

		const progress = scrollY.current / animationScrollDistance;
		return Math.max(0, Math.min(progress, 1));
	});

	const drawFrame = () => {
		if (!frameCanvas || !currentFrameImage) {
			return;
		}

		const context = frameCanvas.getContext('2d');
		if (!context) {
			return;
		}

		const bounds = frameCanvas.getBoundingClientRect();
		const viewportWidth = Math.max(1, Math.round(bounds.width));
		const viewportHeight = Math.max(1, Math.round(bounds.height));
		context.clearRect(0, 0, viewportWidth, viewportHeight);

		const scale = Math.max(
			viewportWidth / currentFrameImage.naturalWidth,
			viewportHeight / currentFrameImage.naturalHeight
		);
		const drawWidth = currentFrameImage.naturalWidth * scale;
		const drawHeight = currentFrameImage.naturalHeight * scale;
		const offsetX = (viewportWidth - drawWidth) / 2;
		const offsetY = (viewportHeight - drawHeight) / 2;

		context.drawImage(currentFrameImage, offsetX, offsetY, drawWidth, drawHeight);
	};

	$effect(() => {
		if (!$isLoading && frameCanvas) {
			frameCanvasResizer?.requestResize();
			drawFrame();
		}
	});

	onMount(() => {
		preloadImages();
		return () => {
			frameCanvasResizer?.stop();
			frameCanvasResizer = null;
		};
	});

	$effect(() => {
		if ($isLoading || !frameCanvas) {
			return;
		}

		frameCanvasResizer = createWindowCanvasResizer({
			canvas: frameCanvas,
			onResize: () => drawFrame()
		});
		frameCanvasResizer.start();
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

	const frameStepClasses = ['pt-40 sm:pt-30', '', 'pt-40 sm:pt-30', 'pt-12'] as const;

	const frameStepContainers = [
		'flex h-full w-full flex-col items-center justify-start gap-0',
		'flex h-full w-full items-center justify-center gap-0',
		'flex h-full w-full flex-col items-center justify-start gap-0',
		'flex h-full w-full flex-col items-center justify-between gap-0'
	] as const;
</script>

{#snippet animatedStep(index: 0 | 1 | 2 | 3)}
	<StepHeading
		title={landingContent.steps[index].title}
		description={landingContent.steps[index].description}
		extraClass={frameStepClasses[index]}
	/>
{/snippet}

{#if $isLoading}
	<div class="flex h-screen w-full flex-col items-center justify-center gap-4 bg-black text-white">
		<h1>Loading...</h1>
		<div class="h-8 w-56 border border-dashed border-white">
			<div class="h-full bg-white" style="width: {$loadProgress}%"></div>
		</div>
		<p>{Math.round($loadProgress)}%</p>

		<a href={resolve('/simple')} class="mx-10 text-center hover:underline">
			Not loading? Please wait a little longer or click me for the simple version of this website.</a
		>

		<img
			src={asset('/loading.webp')}
			alt="loading animation"
			class="absolute right-0 bottom-0 h-32"
		/>
	</div>
{:else}
	<TopBar href="/simple" />
	<div class="relative" style:height={`calc(${animationScrollDistance}px + 100lvh)`}>
		<div class="sticky top-0 w-full overflow-hidden" style="height: 100lvh;">
			<canvas
				bind:this={frameCanvas}
				class="absolute inset-0 z-0 h-full w-full"
				aria-label="scroll animation"
			></canvas>

			<div class="relative z-10 h-full w-full">
				<div
					class="pointer-events-none absolute inset-0 z-10 flex h-screen w-screen flex-col items-center justify-end p-10 pb-20 sm:pb-10"
				>
					<div class="h-fit w-full flex-col items-center-safe justify-start">
						<div
							class="z-10 h-1 origin-left bg-slate-400 transition-transform duration-30 ease-linear will-change-transform"
							style:transform={`scaleX(${scrollProgress})`}
						></div>
					</div>
				</div>

				<!-- Title -->
				{#if isCurrentFrame(0)}
					<div
						class="flex h-full w-full flex-col items-center justify-between gap-0"
						transition:fade={{ duration: 100 }}
					>
						<LandingSection />
					</div>
				{/if}

				<!-- Step 1 -->
				{#if isCurrentFrame(1)}
					<div class={frameStepContainers[0]} transition:fade={{ duration: 100 }}>
						{@render animatedStep(0)}
					</div>
				{/if}

				<!-- Step 2 -->
				{#if isCurrentFrame(2)}
					<div class={frameStepContainers[1]} transition:fade={{ duration: 100 }}>
						<div class="flex flex-col items-center justify-center gap-10 sm:flex-row">
							<img class="landing-step-image" src={`${asset('/ferris.webp')}`} alt="ferris" />
							{@render animatedStep(1)}
						</div>
					</div>
				{/if}

				<!-- Step 3 -->
				{#if isCurrentFrame(3)}
					<div class={frameStepContainers[2]} transition:fade={{ duration: 100 }}>
						{@render animatedStep(2)}
					</div>
				{/if}

				<!-- Step 4 -->
				{#if isCurrentFrame(4)}
					<div class={frameStepContainers[3]} transition:fade={{ duration: 100 }}>
						<StepHeading
							title={landingContent.steps[3].title}
							description={landingContent.steps[3].description}
							extraClass={frameStepClasses[3]}
							descriptionVisible={isCurrentFrame(5)}
						/>
					</div>
				{/if}
			</div>
		</div>
	</div>
	<Faq />
	<Footer showGoUp />
{/if}

<style>
	:global(html) {
		scroll-snap-type: y proximity;
	}
</style>
