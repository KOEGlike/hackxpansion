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

	const controlId = $props.id();
	const labelId = `${controlId}-label`;
	const listboxId = `${controlId}-listbox`;
	let isOpen = $state(false);
	let activeIndex = $state(0);
	let trigger: HTMLButtonElement;
	let optionButtons: HTMLButtonElement[] = [];

	let selectedIndex = $derived(options.findIndex((option) => option.value === value));
	let displayLabel = $derived(selectedIndex >= 0 ? options[selectedIndex].label : placeholder);

	function openDropdown(preferredIndex = selectedIndex) {
		if (disabled) return;
		activeIndex = preferredIndex >= 0 ? preferredIndex : 0;
		isOpen = true;
		requestAnimationFrame(() => optionButtons[activeIndex]?.focus());
	}

	function closeDropdown({ restoreFocus = true } = {}) {
		isOpen = false;
		if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
	}

	function select(index: number) {
		value = options[index].value;
		closeDropdown();
	}

	function moveActive(direction: 1 | -1) {
		activeIndex = (activeIndex + direction + options.length) % options.length;
		optionButtons[activeIndex]?.focus();
	}

	function handleTriggerKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			openDropdown(event.key === 'ArrowUp' ? options.length - 1 : selectedIndex);
		}
	}

	function handleOptionKeydown(event: KeyboardEvent, index: number) {
		switch (event.key) {
			case 'ArrowDown':
				event.preventDefault();
				moveActive(1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				moveActive(-1);
				break;
			case 'Home':
				event.preventDefault();
				activeIndex = 0;
				optionButtons[0]?.focus();
				break;
			case 'End':
				event.preventDefault();
				activeIndex = options.length - 1;
				optionButtons[activeIndex]?.focus();
				break;
			case 'Enter':
			case ' ':
				event.preventDefault();
				select(index);
				break;
			case 'Escape':
				event.preventDefault();
				closeDropdown();
				break;
			case 'Tab':
				closeDropdown({ restoreFocus: false });
				break;
		}
	}
</script>

<div class="relative flex flex-col gap-0.5">
	<span id={labelId}>
		{label}{#if required}<span aria-hidden="true"> *</span>{/if}
	</span>
	<input type="hidden" {name} value={value ?? ''} />

	<button
		bind:this={trigger}
		type="button"
		role="combobox"
		aria-labelledby={`${labelId} ${controlId}`}
		aria-haspopup="listbox"
		aria-controls={listboxId}
		aria-expanded={isOpen}
		aria-required={required}
		id={controlId}
		class="flex w-full cursor-pointer items-center justify-between border border-slate-700 bg-white/70 px-3 py-2 text-left text-sm disabled:cursor-not-allowed disabled:bg-slate-100 disabled:opacity-50 {isOpen
			? 'relative z-20'
			: ''}"
		onclick={() => (isOpen ? closeDropdown() : openDropdown())}
		onkeydown={handleTriggerKeydown}
		{disabled}
	>
		<span class={options[selectedIndex]?.class ?? ''}>{displayLabel}</span>
		<span aria-hidden="true" class="ml-3 text-xs">{isOpen ? '▲' : '▼'}</span>
	</button>

	{#if isOpen}
		<button
			type="button"
			tabindex="-1"
			aria-label={`Close ${label} options`}
			class="fixed inset-0 z-10 h-full w-full cursor-default bg-transparent"
			onclick={() => closeDropdown()}
		></button>

		<ul
			id={listboxId}
			role="listbox"
			aria-labelledby={labelId}
			class="absolute top-[calc(100%+4px)] left-0 z-20 w-full border border-slate-700 bg-white shadow-lg"
		>
			{#each options as option, index (option.value)}
				<li role="presentation">
					<button
						bind:this={optionButtons[index]}
						type="button"
						role="option"
						aria-selected={option.value === value}
						tabindex={index === activeIndex ? 0 : -1}
						class="w-full cursor-pointer px-3 py-2 text-left text-sm transition-colors hover:bg-slate-100 focus:bg-slate-100 focus:outline-none active:bg-slate-200 {option.value ===
						value
							? 'bg-slate-100 font-semibold'
							: ''} {option.class ?? ''}"
						onmouseenter={() => (activeIndex = index)}
						onclick={() => select(index)}
						onkeydown={(event) => handleOptionKeydown(event, index)}
					>
						{option.label}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>
