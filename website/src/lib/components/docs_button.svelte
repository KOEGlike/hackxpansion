<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();
</script>

<div class="fixed top-3 right-3 flex h-fit w-fit flex-col items-end justify-center gap-2">
	<a
		href={resolve('/docs')}
		class="content-box p-2 text-xl hover:content-box-hover active:content-box">Docs</a
	>
	{@render children?.()}
</div>
