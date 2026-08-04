<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { onDestroy, tick } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';

	let { children } = $props();
	let article: HTMLElement;
	let headings = $state<Array<{ id: string; text: string; level: number }>>([]);
	let activeId = $state('');
	let removeScrollListener = () => {};
	let activeIndex = $derived(headings.findIndex((heading) => heading.id === activeId));
	let firstLevel = $derived(Math.min(...headings.map((heading) => heading.level)));

	function headingId(text: string): string {
		return (
			text
				.normalize('NFKD')
				.replace(/[\u0300-\u036f]/g, '')
				.toLowerCase()
				.replace(/[^a-z0-9]+/g, '-')
				.replace(/^-|-$/g, '') || 'section'
		);
	}

	async function buildHeadingTree() {
		await tick();
		removeScrollListener();

		const elements = Array.from(article.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6'));
		const usedIds = new SvelteMap<string, number>();

		headings = elements.map((element) => {
			const baseId = headingId(element.textContent ?? '');
			const occurrence = usedIds.get(baseId) ?? 0;
			const id = occurrence === 0 ? baseId : `${baseId}-${occurrence + 1}`;

			usedIds.set(baseId, occurrence + 1);
			element.id = id;

			return {
				id,
				text: element.textContent?.trim() || 'Untitled section',
				level: Number(element.tagName.slice(1))
			};
		});

		const scrollContainer = article.closest<HTMLElement>('[data-page-scroll]');
		if (!scrollContainer || elements.length === 0) {
			activeId = '';
			return;
		}
		const scrollRoot = scrollContainer;

		function updateActiveHeading() {
			if (scrollRoot.scrollHeight - scrollRoot.scrollTop - scrollRoot.clientHeight < 2) {
				activeId = elements.at(-1)?.id ?? '';
				return;
			}

			const rootTop = scrollRoot.getBoundingClientRect().top;
			let current = elements[0];

			for (const element of elements) {
				if (element.getBoundingClientRect().top - rootTop > 96) break;
				current = element;
			}

			activeId = current.id;
		}

		scrollRoot.addEventListener('scroll', updateActiveHeading, { passive: true });
		removeScrollListener = () => scrollRoot.removeEventListener('scroll', updateActiveHeading);
		updateActiveHeading();

		if (location.hash) {
			document.getElementById(location.hash.slice(1))?.scrollIntoView();
		}
	}

	afterNavigate(() => {
		void buildHeadingTree();
	});

	onDestroy(() => removeScrollListener());
</script>

<div
	class="mx-auto xl:grid xl:max-w-304 xl:grid-cols-[minmax(0,1fr)_14rem] xl:items-start xl:gap-8 xl:px-6"
>
	<main
		bind:this={article}
		class="prose prose-lg mx-auto w-full min-w-0 max-w-4xl p-6 prose-headings:my-2 prose-headings:text-slate-700 prose-headings:underline prose-p:text-slate-700 prose-li:marker:text-slate-700"
	>
		{@render children()}
	</main>

	{#if headings.length > 0}
		<aside
			class="sticky top-6 hidden max-h-[calc(100vh-3rem)] overflow-y-auto py-6 xl:block"
			aria-label="On this page"
		>
			<p class="mb-3 text-xs text-slate-500">
				On this page {Math.max(activeIndex + 1, 1)}/{headings.length}
			</p>
			<ol class="m-0 list-none space-y-1 p-0">
				{#each headings as heading, index (heading.id)}
					<li style:padding-left={`${(heading.level - firstLevel) * 0.75}rem`}>
						<a
							href={`#${heading.id}`}
							class="block truncate text-sm no-underline transition-colors hover:text-slate-900 hover:underline"
							class:font-bold={index === activeIndex}
							class:text-slate-700={index <= activeIndex}
							class:text-slate-400={index > activeIndex}
							aria-current={index === activeIndex ? 'location' : undefined}
						>
							{index === activeIndex ? '> ' : ''}{heading.text}
						</a>
					</li>
				{/each}
			</ol>
		</aside>
	{/if}
</div>

<style>
	:global(.prose h1) {
		font-size: 3rem;
	}

	:global(.prose h2) {
		font-size: 2.25rem;
	}

	:global(.prose h3) {
		font-size: 1.875rem;
	}

	:global(.prose h4) {
		font-size: 1.5rem;
	}

	:global(.prose h5) {
		font-size: 1.25rem;
		font-weight: 600;
		text-decoration-line: underline;
	}

	:global(.prose h6) {
		font-size: 1.125rem;
		font-weight: 600;
		text-decoration-line: underline;
	}

	:global(.prose :is(h1, h2, h3, h4, h5, h6)) {
		scroll-margin-top: 1.5rem;
	}

	:global(.prose :not(pre) > code) {
		border: 1px solid rgb(148 163 184);
		background: rgb(226 232 240);
		padding: 0.1rem 0.35rem;
		font-weight: 400;
		color: rgb(30 41 59);
	}

	:global(.prose :not(pre) > code::before),
	:global(.prose :not(pre) > code::after) {
		content: none;
	}

	:global(.prose pre),
	:global(.prose code) {
		border-radius: 0;
	}

	:global(.prose table) {
		width: 100%;
		border-collapse: collapse;
	}

	:global(.prose :is(th, td)) {
		border: 1px solid rgb(100 116 139);
		background: transparent;
		padding: 0.5rem 0.75rem;
	}
</style>
