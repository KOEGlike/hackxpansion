<script lang="ts">
	import DocsButton from '$lib/components/docs_button.svelte';
	import { resolve } from '$app/paths';

	type ModeSwitchRoute = '/' | '/simple';

	let {
		href,
		label,
		targetText
	}: {
		href: ModeSwitchRoute;
		label: string;
		targetText: string;
	} = $props();
</script>

<DocsButton>
	<a
		class="anchor w-fit text-slate-700 hover:underline"
		href={resolve(href === '/' ? '/' : '/simple')}>{label}</a
	>

	<p class="target text-slate-500">{targetText}</p>
</DocsButton>
