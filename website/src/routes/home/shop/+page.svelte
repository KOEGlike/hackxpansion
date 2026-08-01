<script lang="ts">
	import { asset, resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import { HACKXPANSION_CONSOLE } from '$lib/shop/domain';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();

	function missingApprovalText(item: PageServerData['items'][number]) {
		const missing = [];
		if (item.eligibility.missingModuleDesigns > 0) {
			const count = item.eligibility.missingModuleDesigns;
			missing.push(`${count} more module${count === 1 ? '' : 's'}`);
		}
		if (item.eligibility.missingAppDesigns > 0) {
			const count = item.eligibility.missingAppDesigns;
			missing.push(`${count} more app${count === 1 ? '' : 's'}`);
		}
		return `Need ${missing.join(' and ')} approved`;
	}
</script>

<svelte:head>
	<title>Shop | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
		<div>
			<h1 class="text-4xl font-bold">Shop</h1>
			<p class="text-slate-600">Turn accepted projects into hardware for your next build.</p>
		</div>
		{#if data.signedIn}
			<p class="flex items-center gap-2 text-xl font-bold" aria-label={`Balance: ${data.balance}`}>
				<CoinIcon class="size-6" />
				<span aria-hidden="true">{data.balance}</span>
			</p>
		{/if}
	</header>

	<section aria-label="Shop items" class="flex flex-col gap-4">
		{#if data.items.length === 0}
			<p class="content-box p-5">There are no items in the shop right now.</p>
		{:else}
			<div class="grid gap-5 lg:grid-cols-2">
				{#each data.items as item (item.id)}
					<article class="content-box flex flex-col overflow-hidden">
						<div
							class="flex min-h-44 items-center justify-center border-b border-slate-500 bg-slate-800 p-6 text-white"
						>
							{#if item.imageUrl}
								<img
									src={item.id === HACKXPANSION_CONSOLE.id
										? asset('/shop/console.png')
										: item.imageUrl}
									alt=""
									class="max-h-48 w-full object-contain"
								/>
							{:else}
								<div class="text-center">
									<p class="text-5xl font-bold tracking-tight">HX</p>
									<p class="mt-1 text-sm uppercase tracking-[0.35em]">Console</p>
								</div>
							{/if}
						</div>
						<div class="flex flex-1 flex-col gap-4 p-5">
							<div class="flex items-start justify-between gap-4">
								<h3 class="text-2xl font-bold">{item.name}</h3>
								<p class="flex shrink-0 items-center gap-1 text-xl font-bold">
									<CoinIcon class="size-5" />
									{item.price}
								</p>
							</div>
							<p>{item.description}</p>

							{#if data.signedIn}
								{#if item.canOrder}
									<a
										href={resolve(`/home/shop/order/${encodeURIComponent(item.id)}`)}
										class="mt-auto bg-slate-800 px-4 py-3 text-center font-bold text-white hover:bg-slate-700"
									>
										Continue to order
									</a>
								{:else}
									<button
										class="mt-auto cursor-not-allowed bg-slate-400 px-4 py-3 font-bold text-white"
										disabled
									>
										{!item.unlocked
											? 'Buy a console to unlock'
											: item.eligibility.eligible
												? data.balance >= item.price
													? 'Place order'
													: `Need ${item.price - data.balance} more currency`
												: missingApprovalText(item)}
									</button>
								{/if}
							{:else}
								<form method="post" action={`${resolve('/')}?/signIn`} class="mt-auto">
									<input type="hidden" name="returnTo" value={resolve('/home/shop')} />
									<button
										class="w-full bg-slate-800 px-4 py-3 font-bold text-white hover:bg-slate-700"
									>
										Sign in to order
									</button>
								</form>
							{/if}
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</section>
</main>
