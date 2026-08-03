<script lang="ts">
	import { renderMarkdown } from '$lib/markdown';
	import 'highlight.js/styles/github-dark.css';

	let { text, prose = true }: { text: string; prose?: boolean } = $props();
	let html = $derived(renderMarkdown(text));
</script>

<div class:prose class:prose-sm={prose} class:prose-slate={prose} class:max-w-none={prose}>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -- content is sanitized immediately above -->
	{@html html}
</div>
