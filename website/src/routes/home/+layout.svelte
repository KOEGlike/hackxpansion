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
		{ title: 'First App', href: '/home/docs/quickstart/first-app', indent: true },
		{
			title: 'Basics Of Electronics',
			href: '/home/docs/quickstart/basics-of-electronics',
			indent: true
		},
		{ title: 'Detailed', href: '/home/docs/detailed', indent: false },
		{ title: 'Device', href: '/home/docs/detailed/device', indent: true },
		{ title: 'Pinout', href: '/home/docs/detailed/pinout', indent: true },
		{ title: 'Card', href: '/home/docs/detailed/card', indent: true },
		{ title: 'API', href: '/home/docs/detailed/api', indent: true },
		{ title: 'Shipping', href: '/home/docs/shipping', indent: false },
		{ title: 'Design', href: '/home/docs/shipping/design', indent: true },
		{ title: 'Build', href: '/home/docs/shipping/build', indent: true },
		{ title: 'Helpful Resources', href: '/home/docs/helpful-resources', indent: false }
	] as const;
	const exploreItems = [
		{ title: 'Project Feed', href: '/home/explore' },
		{ title: 'Project Matrix', href: '/home/explore/matrix' }
	] as const;
	const shopItems = [
		{ title: 'Browse', href: '/home/shop' },
		{ title: 'Your Orders', href: '/home/shop/orders' }
	] as const;
	const adminItems = [
		{ title: 'Orders', href: '/home/admin' },
		{ title: 'Projects', href: '/home/admin/projects' },
		{ title: 'Shop Items', href: '/home/admin/shop' },
		{ title: 'Users', href: '/home/admin/users' }
	] as const;

	function isCurrentPage(href: (typeof items)[number]['href']) {
		const pathname = resolve(href);
		return (
			page.url.pathname === pathname ||
			(href !== '/home' && page.url.pathname.startsWith(`${pathname}/`))
		);
	}

	function isExactPage(
		href:
			| (typeof docsItems)[number]['href']
			| (typeof exploreItems)[number]['href']
			| (typeof shopItems)[number]['href']
			| (typeof adminItems)[number]['href']
	) {
		return page.url.pathname === resolve(href);
	}

	function isAdminPage(href: (typeof adminItems)[number]['href']) {
		const pathname = resolve(href);
		return (
			page.url.pathname === pathname ||
			(href !== '/home/admin' && page.url.pathname.startsWith(`${pathname}/`))
		);
	}
</script>

<div class="relative h-screen w-screen overflow-hidden">
	<GridBg />
	<div class="flex flex-row h-full gap-3">
		{#if hidden}
			<button
				class="fixed z-50 m-3 h-fit content-box border-dashed px-2 hover:underline sm:sticky sm:top-3 sm:left-0 sm:writing-vertical-lr"
				onclick={() => (hidden = false)}
				aria-controls="home-sidebar"
				aria-expanded="false"
			>
				Open
			</button>
		{:else}
			<aside
				id="home-sidebar"
				class="fixed z-50 my-3 ml-3 flex h-[calc(100%-1.5rem)] w-fit flex-col justify-between gap-2 overflow-hidden p-3 content-box sm:sticky sm:top-3 sm:left-0"
			>
				<div class="flex min-h-0 w-fit flex-1 flex-col gap-2">
					<button
						class="w-fit shrink-0 hover:underline"
						onclick={() => (hidden = true)}
						aria-controls="home-sidebar"
						aria-expanded="true"
					>
						Close
					</button>
					<hr class="shrink-0" />
					<nav
						aria-label="Account navigation"
						class="flex min-h-0 flex-col gap-1 overflow-x-hidden overflow-y-auto pr-2"
					>
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
							{#if item.href === '/home/shop' && isCurrentPage(item.href)}
								<div class="ml-2 flex flex-col border-l border-slate-400 pl-3">
									{#each shopItems as shopItem (shopItem.href)}
										<a
											href={resolve(shopItem.href)}
											class="text-lg hover:underline"
											class:underline={isExactPage(shopItem.href)}
											aria-current={isExactPage(shopItem.href) ? 'page' : undefined}
										>
											{shopItem.title}
										</a>
									{/each}
								</div>
							{/if}
							{#if item.href === '/home/explore' && isCurrentPage(item.href)}
								<div class="ml-2 flex flex-col border-l border-slate-400 pl-3">
									{#each exploreItems as exploreItem (exploreItem.href)}
										<a
											href={resolve(exploreItem.href)}
											class="text-lg hover:underline"
											class:underline={isExactPage(exploreItem.href)}
											aria-current={isExactPage(exploreItem.href) ? 'page' : undefined}
										>
											{exploreItem.title}
										</a>
									{/each}
								</div>
							{/if}
						{/each}
						{#if data.isAdmin}
							<a
								href={resolve('/home/admin')}
								class="mt-3 text-3xl font-bold hover:underline"
								class:underline={page.url.pathname.startsWith(resolve('/home/admin'))}
								aria-current={page.url.pathname === resolve('/home/admin') ? 'page' : undefined}
							>
								ADMIN
							</a>
							{#if page.url.pathname.startsWith(resolve('/home/admin'))}
								<div class="ml-2 flex flex-col border-l border-slate-400 pl-3">
									{#each adminItems as adminItem (adminItem.href)}
										<a
											href={resolve(adminItem.href)}
											class="text-lg hover:underline"
											class:underline={isAdminPage(adminItem.href)}
											aria-current={isAdminPage(adminItem.href) ? 'page' : undefined}
										>
											{adminItem.title}
										</a>
									{/each}
								</div>
							{/if}
						{/if}
					</nav>
				</div>

				{#if data.user}
					<section aria-label="Account" class="flex shrink-0 flex-row gap-2">
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
				{:else}
					<form
						method="post"
						action={`${resolve('/')}?/signIn`}
						aria-label="Account"
						class="w-full shrink-0"
					>
						<input type="hidden" name="returnTo" value={`${page.url.pathname}${page.url.search}`} />
						<button
							class="content-box w-full cursor-pointer px-4 py-3 font-bold transition-colors hover:bg-slate-700 hover:text-white"
						>
							Sign In/Up
						</button>
					</form>
				{/if}
			</aside>
		{/if}
		<div class="h-full w-full overflow-x-hidden overflow-y-scroll pt-6 sm:pt-0" data-page-scroll>
			{@render children()}
		</div>
	</div>
</div>
