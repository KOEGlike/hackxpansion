<script lang="ts">
	import Markdown from '$lib/components/markdown.svelte';
	import type { PageData } from './$types';

	let { data }: { data: NonNullable<PageData> } = $props();
</script>

<svelte:head>
	<title>{data.title} · Hackxpansion</title>
</svelte:head>

<Markdown text={data.content} prose={false} />
