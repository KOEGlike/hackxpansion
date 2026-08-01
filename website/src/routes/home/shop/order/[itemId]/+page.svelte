<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	const submittedNotes = $derived(
		form && 'notes' in form && typeof form.notes === 'string' ? form.notes : ''
	);
</script>

<svelte:head>
	<title>Order {data.item.name} | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-3xl flex-col gap-6 p-6 text-slate-800">
	<a href={resolve('/home/shop')} class="self-start text-sm font-semibold hover:underline">
		Back to shop
	</a>

	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Confirm order</p>
		<h1 class="text-4xl font-bold">{data.item.name}</h1>
	</header>

	{#if form?.message}
		<p class="border border-red-700 bg-red-100 p-3 text-sm">{form.message}</p>
	{/if}

	<section class="content-box flex flex-col gap-6 p-5" aria-label="Order details">
		<div class="flex items-start justify-between gap-4 border-b border-slate-400 pb-5">
			<div>
				<h2 class="text-xl font-bold">Order summary</h2>
				<p class="mt-1 text-slate-600">{data.item.description}</p>
			</div>
			<p class="flex shrink-0 items-center gap-1 text-xl font-bold">
				<CoinIcon class="size-5" />
				{data.item.price}
			</p>
		</div>

		<form method="post" action="?/order" class="flex flex-col gap-4">
			<label class="flex flex-col gap-2 font-semibold" for="notes">
				Notes for the fulfiller <span class="text-sm font-normal text-slate-500">(optional)</span>
				<textarea
					id="notes"
					name="notes"
					rows="5"
					maxlength="2000"
					class="border border-slate-700 bg-white/80 p-3 font-normal"
					value={submittedNotes}
					placeholder="Add any details the fulfiller should know"></textarea>
			</label>

			<div class="flex flex-col-reverse gap-3 sm:flex-row sm:items-center sm:justify-end">
				<a href={resolve('/home/shop')} class="px-4 py-3 text-center font-semibold hover:underline">
					Cancel
				</a>
				<button
					class="bg-slate-800 px-5 py-3 font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
					disabled={!data.item.canOrder}
				>
					{data.item.canOrder ? 'Place order' : 'Order is not currently available'}
				</button>
			</div>
		</form>
	</section>
</main>
