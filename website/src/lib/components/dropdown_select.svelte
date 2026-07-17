<script lang="ts">
	let {
		name,
		label,
		options,
		value = $bindable(null),
		disabled = false,
		required = false,
		placeholder = 'Select...'
	}: {
		name: string;
		label: string;
		options: { value: string | null; label: string; class?: string }[];
		value: string | null;
		disabled?: boolean;
		required?: boolean;
		placeholder?: string;
	} = $props();

	let isOpen = $state(false);

	function select(newValue: string | null) {
		value = newValue;
		isOpen = false;
	}

	function displayLabel() {
		if (value == null) return placeholder;
		return options.find((o) => o.value === value)?.label ?? placeholder;
	}
</script>

<div class="flex flex-col gap-0.5 relative">
	<span>
		{label}{#if required} *{/if}
	</span>
	<input type="hidden" {name} value={value ?? ''} {required} />

	<button
		type="button"
		class="flex w-full items-center justify-between border border-slate-700 bg-white/70 px-3 py-2 text-left text-sm cursor-pointer disabled:cursor-not-allowed disabled:bg-slate-100 disabled:opacity-50 {isOpen ? 'relative z-20' : ''}"
		onclick={() => (isOpen = !isOpen)}
		{disabled}
	>
		<span>{displayLabel()}</span>
	</button>

	{#if isOpen}
		<button
			type="button"
			tabindex="-1"
			aria-label="Close dropdown"
			class="fixed inset-0 z-10 h-full w-full cursor-default"
			onclick={() => (isOpen = false)}
		></button>

		<ul
			class="absolute top-[calc(100%+4px)] left-0 z-20 w-full border border-slate-700 bg-white shadow-lg"
		>
			{#each options as option (option.value)}
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm cursor-pointer hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 transition-colors {option.class ?? ''}"
						onclick={() => select(option.value)}
					>
						{option.label}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>
