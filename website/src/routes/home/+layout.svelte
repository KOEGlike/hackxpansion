<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import GridBg from '$lib/components/grid_bg.svelte';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import { MediaQuery } from 'svelte/reactivity';

	let { data, children } = $props();

	// eslint-disable-next-line svelte/prefer-writable-derived -- users can override the viewport default
	let hidden = $state(false);
	const smallViewport = new MediaQuery('(max-width: 639px)', false);

	$effect(() => {
		hidden = smallViewport.current;
	});

	const items = [
		{ title: 'Home', href: '/home' },
		{ title: 'Projects', href: '/home/projects' },
		{ title: 'Shop', href: '/home/shop' },
		{ title: 'Docs', href: '/home/docs' },
		{ title: 'Explore', href: '/home/explore' },
		{ title: 'Settings', href: '/home/settings' }
	] as const;
	const docsItems = [
		{ title: 'Getting Started', href: '/home/docs/quickstart', indent: false },
		{ title: 'First Card', href: '/home/docs/quickstart/first-card', indent: true },
		{ title: 'First Driver', href: '/home/docs/quickstart/first-driver', indent: true },
		{ title: 'Detailed', href: '/home/docs/detailed', indent: false },
		{ title: 'Device', href: '/home/docs/detailed/device', indent: true },
		{ title: 'Card', href: '/home/docs/detailed/card', indent: true },
		{ title: 'API', href: '/home/docs/detailed/api', indent: true }
	] as const;

	function isCurrentPage(href: (typeof items)[number]['href']) {
		const pathname = resolve(href);
		return (
			page.url.pathname === pathname ||
			(href !== '/home' && page.url.pathname.startsWith(`${pathname}/`))
		);
	}

	function isExactPage(href: (typeof docsItems)[number]['href']) {
		return page.url.pathname === resolve(href);
	}
</script>

<div class="relative h-screen w-screen overflow-hidden">
	<GridBg />
	<div class="flex flex-row h-full gap-3">
		{#if hidden}
			<button
				class="m-3 fixed h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
				aria-controls="home-sidebar"
				aria-expanded="false"
			>
				Open
			</button>
		{:else}
			<aside
				id="home-sidebar"
				class="fixed flex h-[calc(100%-1.5rem)] w-fit flex-col content-box p-3 sm:sticky sm:top-3 sm:left-0 justify-between my-3 ml-3"
			>
				<div class="flex h-fit w-fit flex-col gap-2">
					<button
						class="w-fit hover:underline"
						onclick={() => (hidden = true)}
						aria-controls="home-sidebar"
						aria-expanded="true"
					>
						Close
					</button>
					<hr />
					<nav aria-label="Account navigation" class="flex flex-col gap-1">
						{#each items as item (item.href)}
							<a
								href={resolve(item.href)}
								class="text-3xl hover:underline mr-30"
								class:underline={isCurrentPage(item.href)}
								aria-current={isCurrentPage(item.href) ? 'page' : undefined}
							>
								{item.title}
							</a>
							{#if item.href === '/home/docs' && isCurrentPage(item.href)}
								<div class="ml-2 flex flex-col border-l border-slate-400 pl-3">
									{#each docsItems as docsItem (docsItem.href)}
										<a
											href={resolve(docsItem.href)}
											class="text-lg hover:underline"
											class:ml-4={docsItem.indent}
											class:underline={isExactPage(docsItem.href)}
											aria-current={isExactPage(docsItem.href) ? 'page' : undefined}
										>
											{docsItem.title}
										</a>
									{/each}
								</div>
							{/if}
						{/each}
					</nav>
				</div>

				{#if data.user}
					<section aria-label="Account" class="flex flex-row gap-2">
						<img src={data.user.image} alt="" class="size-20" />
						<div class="flex flex-col py-3 justify-between">
							<p class="text-xl">{data.user.name}</p>
							<p
								class="flex items-center gap-1 font-semibold"
								aria-label={`Currency balance: ${data.user.currency}`}
							>
								<CoinIcon class="size-5" />
								<span aria-hidden="true">{data.user.currency}</span>
							</p>
							<form method="post" action={`${resolve('/home')}?/signOut`}>
								<button class="w-fit hover:underline cursor-pointer">Sign out</button>
							</form>
						</div>
					</section>
				{/if}
			</aside>
		{/if}
		<div class="overflow-x-hidden overflow-y-scroll h-full w-full">
			{@render children()}
		</div>
	</div>
</div>
