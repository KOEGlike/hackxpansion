<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'isomorphic-dompurify';

	let { text, prose = true }: { text: string; prose?: boolean } = $props();
	let html = $derived(DOMPurify.sanitize(marked.parse(text, { async: false }) as string));
</script>

<div class:prose class:prose-sm={prose} class:prose-slate={prose} class:max-w-none={prose}>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -- content is sanitized immediately above -->
	{@html html}
</div>
