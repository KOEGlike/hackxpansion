<script lang="ts">
	import type { CatalogItemFormValues } from '$lib/shop/catalog';

	let {
		values,
		mode
	}: {
		values: CatalogItemFormValues;
		mode: 'create' | 'update';
	} = $props();

	const prefix = $derived(mode === 'create' ? 'new-item' : `item-${values.id}`);
</script>

<form method="post" action={`?/${mode}`} class="flex flex-col gap-4">
	<div class="grid gap-4 md:grid-cols-2">
		<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-id`}>
			Item ID
			<input
				id={`${prefix}-id`}
				name="id"
				value={values.id}
				required
				readonly={mode === 'update'}
				maxlength="100"
				pattern="[a-z0-9]+(?:-[a-z0-9]+)*"
				class="border border-slate-700 bg-white/80 p-2 font-normal read-only:bg-slate-200"
			/>
			<span class="font-normal text-slate-500">Lowercase letters, numbers, and hyphens.</span>
		</label>

		<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-name`}>
			Name
			<input
				id={`${prefix}-name`}
				name="name"
				value={values.name}
				required
				maxlength="100"
				class="border border-slate-700 bg-white/80 p-2 font-normal"
			/>
		</label>
	</div>

	<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-description`}>
		Description
		<textarea
			id={`${prefix}-description`}
			name="description"
			value={values.description}
			required
			rows="3"
			maxlength="2000"
			class="border border-slate-700 bg-white/80 p-2 font-normal"></textarea>
	</label>

	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
		<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-price`}>
			Price in coins
			<input
				id={`${prefix}-price`}
				name="price"
				type="number"
				value={values.price}
				required
				min="0"
				max="2147483647"
				step="1"
				class="border border-slate-700 bg-white/80 p-2 font-normal"
			/>
		</label>

		<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-sort-order`}>
			Sort order
			<input
				id={`${prefix}-sort-order`}
				name="sortOrder"
				type="number"
				value={values.sortOrder}
				required
				min="-2147483647"
				max="2147483647"
				step="1"
				class="border border-slate-700 bg-white/80 p-2 font-normal"
			/>
		</label>

		<label class="flex items-center gap-2 self-end py-3 text-sm font-semibold">
			<input name="active" type="checkbox" checked={values.active} class="size-5" />
			Visible in shop
		</label>
	</div>

	<label class="flex flex-col gap-1 text-sm font-semibold" for={`${prefix}-image-url`}>
		Image URL <span class="font-normal text-slate-500">(optional)</span>
		<input
			id={`${prefix}-image-url`}
			name="imageUrl"
			value={values.imageUrl}
			maxlength="2000"
			placeholder="https://example.com/item.webp or /shop/item.webp"
			class="border border-slate-700 bg-white/80 p-2 font-normal"
		/>
	</label>

	<button class="self-start bg-slate-800 px-5 py-2 font-bold text-white hover:bg-slate-700">
		{mode === 'create' ? 'Add item' : 'Save changes'}
	</button>
</form>
