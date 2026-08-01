<script lang="ts">
	import Markdown from '$lib/components/markdown.svelte';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
</script>

<svelte:head>
	<title>{data.title} · HackXPansion</title>
</svelte:head>

<Markdown text={data.content} prose={false} />
