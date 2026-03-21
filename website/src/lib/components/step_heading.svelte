<script lang="ts">
	import { fade } from 'svelte/transition';

	let {
		title,
		description,
		centered = true,
		extraClass = '',
		descriptionVisible = true
	}: {
		title: string;
		description: string;
		centered?: boolean;
		extraClass?: string;
		descriptionVisible?: boolean;
	} = $props();
</script>

<div
	class={`landing-step-heading ${centered ? 'items-center text-center' : ''} ${extraClass}`.trim()}
>
	<h1 class="landing-step-title">{title}</h1>
	{#if descriptionVisible}
		<h2 class="w-85 landing-step-description" transition:fade={{ duration: 100 }}>{description}</h2>
	{/if}
</div>
