<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import GridBg from '$lib/components/grid_bg.svelte';
	import { MediaQuery } from 'svelte/reactivity';

	// eslint-disable-next-line svelte/prefer-writable-derived -- users can override the viewport default
	let hidden = $state(false);
	const smallViewport = new MediaQuery('(max-width: 639px)', false);

	$effect(() => {
		hidden = smallViewport.current;
	});

	let { children } = $props();
</script>

<a
	class="fixed top-3 right-3 z-50 h-12.5 w-12.5 content-box p-0.5"
	href={resolve('/')}
	aria-label="HackXPansion home"
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
	<GridBg />
	<div
		class="relative z-10 flex h-fit w-screen flex-row p-3 {!hidden
			? ' gap-2 sm:gap-4'
			: 'gap-1 sm:gap-2'}"
	>
		{#if !hidden}
			<aside
				id="docs-sidebar"
				class="fixed flex h-fit w-fit flex-col content-box p-2 sm:sticky sm:top-3 sm:left-0"
			>
				<button
					class="w-fit hover:underline"
					onclick={() => (hidden = true)}
					aria-controls="docs-sidebar"
					aria-expanded="true"
				>
					Close
				</button>
				<nav aria-label="Documentation" class="flex flex-col">
					<a
						class="text-2xl hover:underline"
						href={resolve('/docs')}
						class:underline={page.url.pathname === resolve('/docs')}
						aria-current={page.url.pathname === resolve('/docs') ? 'page' : undefined}
					>
						Documentation
					</a>

					<a
						href={resolve('/docs/quickstart')}
						class="text-lg hover:underline"
						class:underline={page.url.pathname === resolve('/docs/quickstart')}
						aria-current={page.url.pathname === resolve('/docs/quickstart') ? 'page' : undefined}
					>
						Getting Started
					</a>

					<a
						href={resolve('/docs/detailed')}
						class="text-lg hover:underline"
						class:underline={page.url.pathname === resolve('/docs/detailed')}
						aria-current={page.url.pathname === resolve('/docs/detailed') ? 'page' : undefined}
					>
						Detailed
					</a>
					<a
						href={resolve('/docs/detailed/card')}
						class="indent-4 hover:underline"
						class:underline={page.url.pathname === resolve('/docs/detailed/card')}
						aria-current={page.url.pathname === resolve('/docs/detailed/card') ? 'page' : undefined}
					>
						Card
					</a>
				</nav>
			</aside>
		{:else}
			<button
				class="fixed h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
				aria-controls="docs-sidebar"
				aria-expanded="false"
			>
				Open
			</button>
		{/if}
		<main
			class="prose prose-lg max-w-none pt-8 sm:max-w-7/12 sm:pt-0 prose-headings:my-2 prose-headings:text-slate-700 prose-p:text-slate-700"
		>
			{@render children()}
		</main>
	</div>
</div>
