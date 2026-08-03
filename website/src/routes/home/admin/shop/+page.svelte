<script lang="ts">
	import AdminShopItemForm from '$lib/components/admin_shop_item_form.svelte';
	import type { CatalogItemFormValues } from '$lib/shop/catalog';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	const emptyItem: CatalogItemFormValues = {
		id: '',
		name: '',
		description: '',
		price: '0',
		imageUrl: '',
		sortOrder: '0',
		active: true
	};

	function valuesForCreate() {
		return form && !form.success && form.action === 'create' && 'values' in form && form.values
			? form.values
			: emptyItem;
	}

	function valuesForItem(item: PageServerData['items'][number]): CatalogItemFormValues {
		if (
			form &&
			!form.success &&
			form.action === 'update' &&
			form.itemId === item.id &&
			'values' in form &&
			form.values
		) {
			return form.values;
		}
		return {
			id: item.id,
			name: item.name,
			description: item.description,
			price: String(item.price),
			imageUrl: item.imageUrl ?? '',
			sortOrder: String(item.sortOrder),
			active: item.active
		};
	}
</script>

<svelte:head>
	<title>Shop Items | Hackxpansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-6xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin</p>
		<h1 class="text-4xl font-bold">Shop items</h1>
		<p class="text-slate-600">Add and edit items that unlock after a console purchase.</p>
		<p class="mt-2 text-sm text-slate-500">
			The Hackxpansion Console is permanent and managed in code, so it cannot be edited here.
		</p>
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

	<section class="content-box p-5" aria-labelledby="new-item-heading">
		<h2 id="new-item-heading" class="mb-5 text-2xl font-bold">Add item</h2>
		<AdminShopItemForm values={valuesForCreate()} mode="create" />
	</section>

	<section class="flex flex-col gap-4" aria-labelledby="existing-items-heading">
		<h2 id="existing-items-heading" class="text-2xl font-bold">Existing items</h2>
		{#if data.items.length === 0}
			<p class="content-box p-5">No additional shop items have been added.</p>
		{:else}
			{#each data.items as item (item.id)}
				<article class="content-box p-5">
					<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
						<h3 class="text-xl font-bold">{item.name}</h3>
						<span
							class="px-2 py-1 text-xs font-bold uppercase"
							class:bg-green-200={item.active}
							class:bg-slate-300={!item.active}
						>
							{item.active ? 'Active' : 'Hidden'}
						</span>
					</div>
					<AdminShopItemForm values={valuesForItem(item)} mode="update" />
				</article>
			{/each}
		{/if}
	</section>
</main>
