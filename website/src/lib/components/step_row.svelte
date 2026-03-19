<script lang="ts">
	import StepHeading from '$lib/components/step_heading.svelte';

	let {
		title,
		description,
		imageSrc,
		imageAlt,
		reversed = false
	}: {
		title: string;
		description: string;
		imageSrc: string;
		imageAlt: string;
		reversed?: boolean;
	} = $props();
</script>

<div class={`adaptive-container ${reversed ? 'sm:flex-row-reverse' : ''}`.trim()}>
	<StepHeading {title} {description} />
	<img class="landing-step-image" src={imageSrc} alt={imageAlt} />
</div>
