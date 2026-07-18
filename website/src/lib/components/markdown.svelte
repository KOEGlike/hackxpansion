<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'isomorphic-dompurify';

	let { text }: { text: string } = $props();
	let html = $derived(DOMPurify.sanitize(marked.parse(text, { async: false }) as string));
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -- content is sanitized immediately above -->
<div class="prose prose-sm prose-slate max-w-none">{@html html}</div>
