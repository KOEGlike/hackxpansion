<script lang="ts">
	import { asset } from '$app/paths';
	import LandingSection from '$lib/components/landing_section.svelte';
	import TopBar from '$lib/components/top_bar.svelte';
	import StepRow from '$lib/components/step_row.svelte';
	import Footer from '$lib/components/footer.svelte';
	import { landingContent } from '$lib/content/content';

	const simpleSteps = landingContent.steps.map((step, index) => {
		const media = landingContent.simpleStepMedia[index];
		return {
			...step,
			...media,
			resolvedImageSrc: asset(media.imageSrc)
		};
	});
</script>

<TopBar href="/" />

<div class="flex h-fit w-full flex-col items-center justify-start gap-15 px-10 pb-30 sm:gap-30">
	<div
		class="flex h-screen w-screen flex-col items-center justify-between gap-0 bg-cover bg-center"
		style:background-image="url({asset(`/renders/0000.webp`)})"
	>
		<LandingSection />
	</div>
	{#each simpleSteps as step (step.title)}
		<hr class="landing-section-divider" />
		<StepRow
			title={step.title}
			description={step.description}
			imageSrc={step.resolvedImageSrc}
			imageAlt={step.imageAlt}
			reversed={step.reversed}
		/>
	{/each}
</div>
<Footer />
