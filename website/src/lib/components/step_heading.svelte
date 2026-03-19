<script lang="ts">
	let {
		title,
		description,
		centered = true,
		extraClass = ''
	}: {
		title: string;
		description: string;
		centered?: boolean;
		extraClass?: string;
	} = $props();
</script>

<div
	class={`landing-step-heading ${centered ? 'items-center text-center' : ''} ${extraClass}`.trim()}
>
	<h1 class="landing-step-title">{title}</h1>
	<h2 class="w-85 landing-step-description">{description}</h2>
</div>
