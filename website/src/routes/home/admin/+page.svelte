<script lang="ts">
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });
</script>

<svelte:head>
	<title>Orders | HackXPansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-6xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin</p>
		<h1 class="text-4xl font-bold">Shop orders</h1>
		<p class="text-slate-600">Review notes and mark queued orders as fulfilled.</p>
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

	{#if data.orders.length === 0}
		<p class="content-box p-5">No shop orders have been placed.</p>
	{:else}
		<section class="flex flex-col gap-4" aria-label="All shop orders">
			{#each data.orders as order (order.id)}
				<article class="content-box p-5">
					<div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
						<div>
							<div class="flex flex-wrap items-center gap-2">
								<h2 class="text-xl font-bold">{order.itemName}</h2>
								<span
									class="px-2 py-1 text-xs font-bold uppercase"
									class:bg-amber-200={order.status === 'in_queue'}
									class:bg-green-200={order.status === 'fulfilled'}
								>
									{order.status === 'in_queue' ? 'In queue' : 'Fulfilled'}
								</span>
							</div>
							<p class="mt-1">{order.userName} · {order.userEmail}</p>
							<p class="text-sm text-slate-600">{dateFormatter.format(order.createdAt)}</p>
						</div>
						<p class="flex items-center gap-1 font-bold"><CoinIcon /> {order.pricePaid}</p>
					</div>

					<div class="mt-4 grid gap-4 lg:grid-cols-2">
						<div>
							<p class="text-xs font-bold uppercase text-slate-500">Notes from user</p>
							<p class="mt-1 whitespace-pre-wrap">{order.notes ?? 'No notes provided.'}</p>
						</div>
						{#if order.status === 'in_queue'}
							<form method="post" action="?/fulfill" class="flex flex-col gap-2">
								<input type="hidden" name="orderId" value={order.id} />
								<label class="text-xs font-bold uppercase" for={`message-${order.id}`}>
									Message to user <span class="font-normal text-slate-500">(optional)</span>
								</label>
								<textarea
									id={`message-${order.id}`}
									name="message"
									rows="3"
									maxlength="2000"
									class="border border-slate-700 bg-white/80 p-2 font-normal"></textarea>
								<button class="self-start bg-slate-800 px-4 py-2 text-white hover:bg-slate-700">
									Mark fulfilled
								</button>
							</form>
						{:else}
							<div>
								<p class="text-xs font-bold uppercase text-slate-500">Fulfillment message</p>
								<p class="mt-1 text-sm text-slate-600">
									Fulfilled by {order.fulfillerName ??
										order.fulfilledByUserId ??
										'an admin'}{order.fulfilledAt
										? ` on ${dateFormatter.format(order.fulfilledAt)}`
										: ''}.
								</p>
								<p class="mt-1 whitespace-pre-wrap">
									{order.fulfillmentMessage ?? 'No message provided.'}
								</p>
							</div>
						{/if}
					</div>
				</article>
			{/each}
		</section>
	{/if}
</main>
