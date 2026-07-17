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
		{ title: 'Shop', href: '/home/shop' },
		{ title: 'Docs', href: '/docs' },
		{ title: 'Explore', href: '/home/explore' },
		{ title: 'Settings', href: '/home/settings' }
	] as const;
</script>

<div class="relative h-screen w-screen overflow-hidden">
	<GridBg />
	<div class="flex flex-row h-full gap-3">
		{#if hidden}
			<button
				class="m-3 fixed h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
			>
				Open
			</button>
		{:else}
			<div
				class="fixed flex h-[calc(100%-1.5rem)] w-fit flex-col content-box p-3 sm:sticky sm:top-3 sm:left-0 justify-between my-3 ml-3"
			>
				<div class="flex h-fit w-fit flex-col gap-2">
					<button class="w-fit hover:underline" onclick={() => (hidden = true)}>Close</button>
					<hr />
					<div class="flex flex-col gap-1">
						{#each items as item (item.href)}
							<a href={resolve(item.href)} class="text-3xl hover:underline mr-30">{item.title}</a>
						{/each}
					</div>
				</div>

				{#if data.user}
					<div class="flex flex-row gap-2">
						<!-- svelte-ignore a11y_img_redundant_alt -->
						<img src={data.user.image} alt="Profile picture" class="size-20" />
						<div class="flex flex-col py-3 justify-between">
							<p class="text-xl">{data.user.name}</p>
							<p>{data.user.pronouns}</p>
						</div>
					</div>
				{/if}
			</div>
		{/if}
		<div class="overflow-x-hidden overflow-y-scroll h-full w-full">
			{@render children()}
		</div>
	</div>
</div>
