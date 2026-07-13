<script lang="ts">
	import { resolve } from '$app/paths';
	import GridBg from '$lib/components/grid_bg.svelte';
	import { onMount } from 'svelte';

	let { data, children } = $props();

	onMount(() => {
		let vw = Math.max(document.documentElement.clientWidth || 0, window.innerWidth || 0);
		if (vw < 640) {
			hidden = true;
		}
	});

	let hidden = $state(false);

	const items = [
		{ title: 'Home', href: '/home' },
		{ title: 'Projects', href: '/home/projects' },
		{ title: 'Docs', href: '/docs' },
		{ title: 'Explore', href: '/home/explore' },
		{ title: 'Settings', href: '/home/settings' }
	] as const;
</script>

<div class="relative h-screen w-screen">
	<GridBg />
	<div class="flex flex-row p-3 h-full">
		{#if hidden}
			<button
				class="fixed h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
			>
				Open
			</button>
		{:else}
			<div
				class="fixed flex h-full w-fit flex-col content-box p-1 sm:sticky sm:top-3 sm:left-0 justify-between"
			>
				<div class="flex h-fit w-fit flex-col gap-2">
					<button class="w-fit hover:underline" onclick={() => (hidden = true)}>Close</button>
					<hr />
					<div class="flex flex-col gap-1">
						{#each items as item (item.href)}
							<a href={resolve(item.href)} class="text-xl hover:underline mr-20">{item.title}</a>
						{/each}
					</div>
				</div>
				<div>sdfasdfs</div>
			</div>
		{/if}
	</div>
</div>
