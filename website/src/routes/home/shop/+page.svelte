<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	function requirementText(item: PageServerData['items'][number]) {
		const parts = [];
		if (item.requiredModuleDesigns > 0) {
			parts.push(`${item.requiredModuleDesigns} accepted module designs`);
		}
		if (item.requiredAppDesigns > 0) {
			parts.push(
				`${item.requiredAppDesigns} accepted app design${item.requiredAppDesigns === 1 ? '' : 's'}`
			);
		}
		return parts.length > 0 ? parts.join(' and ') : 'No project requirements';
	}

	function submittedNotes(itemId: string) {
		return form &&
			'itemId' in form &&
			form.itemId === itemId &&
			'notes' in form &&
			typeof form.notes === 'string'
			? form.notes
			: '';
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

	{#if form?.message}
		<p
			class="border p-3 text-sm"
			class:border-green-700={form.success}
			class:bg-green-100={form.success}
			class:border-red-700={!form.success}
			class:bg-red-100={!form.success}
		>
			{form.message}
		</p>
	{/if}

	{#if data.signedIn}
		<section class="content-box grid gap-4 p-5 sm:grid-cols-2" aria-labelledby="shop-progress">
			<div>
				<h2 id="shop-progress" class="text-xl font-bold">Your unlock progress</h2>
				<p class="mt-1 text-sm text-slate-600">Designs count once Ari accepts them.</p>
			</div>
			<div class="grid grid-cols-2 gap-3 text-center">
				<div class="border border-slate-400 bg-white/40 p-3">
					<p class="text-2xl font-bold">{data.progress.moduleDesigns}</p>
					<p class="text-xs uppercase tracking-wide">Modules</p>
				</div>
				<div class="border border-slate-400 bg-white/40 p-3">
					<p class="text-2xl font-bold">{data.progress.appDesigns}</p>
					<p class="text-xs uppercase tracking-wide">Apps</p>
				</div>
			</div>
		</section>
	{/if}

	<section aria-labelledby="shop-items" class="flex flex-col gap-4">
		<h2 id="shop-items" class="text-2xl font-bold">Available items</h2>
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
								<img src={item.imageUrl} alt="" class="max-h-48 w-full object-contain" />
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
							<p class="border-l-2 border-slate-500 pl-3 text-sm text-slate-600">
								Requires {requirementText(item)}.
							</p>

							{#if data.signedIn}
								<form method="post" action="?/order" class="mt-auto flex flex-col gap-3">
									<input type="hidden" name="itemId" value={item.id} />
									<label class="flex flex-col gap-1 text-sm font-semibold" for={`notes-${item.id}`}>
										Notes for the fulfiller <span class="font-normal text-slate-500"
											>(optional)</span
										>
										<textarea
											id={`notes-${item.id}`}
											name="notes"
											rows="3"
											maxlength="2000"
											class="border border-slate-700 bg-white/80 p-2 font-normal"
											value={submittedNotes(item.id)}></textarea>
									</label>
									<button
										class="bg-slate-800 px-4 py-3 font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
										disabled={!item.canOrder}
									>
										{item.eligibility.eligible
											? data.balance >= item.price
												? 'Place order'
												: `Need ${item.price - data.balance} more currency`
											: 'Project requirements not met'}
									</button>
								</form>
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
