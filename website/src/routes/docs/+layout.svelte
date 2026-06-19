<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import DocsBg from '$lib/components/docs_bg.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		let vw = Math.max(document.documentElement.clientWidth || 0, window.innerWidth || 0);
		if (vw < 640) {
			hidden = true;
		}
	});

	let hidden = $state(false);

	let { children } = $props();
</script>

<a
	class="fixed top-3 right-3 z-50 h-12.5 w-12.5 content-box p-0.5"
	href={resolve('/')}
	aria-label="Home Page"
>
	<svg
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 -960 960 960"
		class="h-full w-full fill-slate-700"
	>
		<path
			d="M220-180h150v-250h220v250h150v-390L480-765 220-570v390Zm-60 60v-480l320-240 320 240v480H530v-250H430v250H160Zm320-353Z"
		/>
	</svg>
</a>

<div class="relative min-h-screen w-screen">
	<DocsBg />
	<div
		class="relative z-10 flex h-fit w-screen flex-row p-3 {!hidden
			? ' gap-2 sm:gap-4'
			: 'gap-1 sm:gap-2'}"
	>
		{#if !hidden}
			<div class="fixed flex h-fit w-fit flex-col content-box p-2 sm:sticky sm:top-3 sm:left-0">
				<button class="w-fit hover:underline" onclick={() => (hidden = true)}>Close</button>
				<a
					class="text-2xl hover:underline"
					href={resolve('/docs')}
					class:underline={page.url.pathname === resolve('/docs')}
				>
					Documentation
				</a>

				<a
					href={resolve('/docs/quickstart')}
					class="text-lg hover:underline"
					class:underline={page.url.pathname === resolve('/docs/quickstart')}
				>
					Getting Started
				</a>

				<a
					href={resolve('/docs/quickstart/first-card')}
					class="indent-4 hover:underline"
					class:underline={page.url.pathname === resolve('/docs/quickstart/first-card')}
				>
					First card
				</a>
				<a
					href={resolve('/docs/quickstart/first-driver')}
					class="indent-4 hover:underline"
					class:underline={page.url.pathname === resolve('/docs/quickstart/first-driver')}
				>
					First Driver
				</a>

				<a
					href={resolve('/docs/detailed')}
					class="text-lg hover:underline"
					class:underline={page.url.pathname === resolve('/docs/detailed')}
				>
					Detailed
				</a>
				<a
					href={resolve('/docs/detailed/card')}
					class="indent-4 hover:underline"
					class:underline={page.url.pathname === resolve('/docs/detailed/card')}
				>
					Card
				</a>
				<a
					href={resolve('/docs/detailed/api')}
					class="indent-4 hover:underline"
					class:underline={page.url.pathname === resolve('/docs/detailed/api')}
				>
					Driver API
				</a>
				<a
					href={resolve('/docs/detailed/device')}
					class="indent-4 hover:underline"
					class:underline={page.url.pathname === resolve('/docs/detailed/device')}
				>
					Device
				</a>
			</div>
		{:else}
			<button
				class="fixed h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
			>
				Open
			</button>
		{/if}
		<div
			class="prose prose-lg max-w-none pt-8 sm:max-w-7/12 sm:pt-0 prose-headings:my-2 prose-headings:text-slate-700 prose-p:text-slate-700"
		>
			{@render children()}
		</div>
	</div>
</div>
