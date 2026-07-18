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
	<h2 class="landing-step-title">{title}</h2>
	{#if descriptionVisible}
		<p
			class="w-[calc(100%-2rem)] max-w-100 landing-step-description"
			transition:fade={{ duration: 100 }}
		>
			{description}
		</p>
	{/if}
</div>
