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

<style>
	:global(pre.highlighted-code) {
		background: #0d1117;
		padding: 0;
	}

	:global(pre.highlighted-code > code.hljs) {
		border: 0;
		margin: 0;
	}
</style>
