<script lang="ts">
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import AccountRequired from '$lib/components/account_required.svelte';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium' });
</script>

<svelte:head>
	<title>Your orders | HackXPansion</title>
</svelte:head>

{#if !data.signedIn}
	<AccountRequired message="You need an account to view your HackXPansion orders." />
{:else}
	<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
		<header>
			<h1 class="text-4xl font-bold">Your orders</h1>
			<p class="text-slate-600">Track rewards you have ordered from the shop.</p>
		</header>

		{#if data.orders.length === 0}
			<p class="content-box p-5">You have not placed an order yet.</p>
		{:else}
			<section class="flex flex-col gap-4" aria-label="Order history">
				{#each data.orders as order (order.id)}
					<article class="content-box p-5">
						<div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
							<div>
								<h2 class="text-xl font-bold">{order.itemName}</h2>
								<p class="text-sm text-slate-600">
									Ordered {dateFormatter.format(order.createdAt)}
								</p>
							</div>
							<div class="flex items-center gap-3">
								<span class="flex items-center gap-1 font-bold"><CoinIcon /> {order.pricePaid}</span
								>
								<span
									class="px-2 py-1 text-xs font-bold uppercase"
									class:bg-amber-200={order.status === 'in_queue'}
									class:bg-green-200={order.status === 'fulfilled'}
								>
									{order.status === 'in_queue' ? 'In queue' : 'Fulfilled'}
								</span>
							</div>
						</div>
						{#if order.notes}
							<p class="mt-4 text-sm"><strong>Your notes:</strong> {order.notes}</p>
						{/if}
						{#if order.status === 'fulfilled'}
							<p class="mt-4 text-sm text-slate-600">
								Fulfilled by {order.fulfillerName ??
									order.fulfilledByUserId ??
									'an admin'}{order.fulfilledAt
									? ` on ${dateFormatter.format(order.fulfilledAt)}`
									: ''}.
							</p>
						{/if}
						{#if order.fulfillmentMessage}
							<div class="mt-4 border-l-2 border-green-700 bg-green-50 p-3">
								<p class="text-xs font-bold uppercase text-green-900">Message from the fulfiller</p>
								<p class="mt-1">{order.fulfillmentMessage}</p>
							</div>
						{/if}
					</article>
				{/each}
			</section>
		{/if}
	</main>
{/if}
