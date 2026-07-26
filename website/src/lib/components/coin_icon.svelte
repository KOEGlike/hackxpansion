<script lang="ts">
	let { class: className = 'size-4' }: { class?: string } = $props();
</script>

<svg
	class={className}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.75"
	aria-hidden="true"
>
	<circle cx="12" cy="12" r="9" class="fill-amber-300 stroke-amber-800" />
	<circle cx="12" cy="12" r="5.5" class="stroke-amber-700" />
	<path
		d="M14.5 9.5c-.6-.7-1.4-1-2.5-1-1.4 0-2.5.7-2.5 1.8 0 2.9 5 1.1 5 3.9 0 1.1-1.1 1.8-2.5 1.8-1.1 0-2-.4-2.7-1.1M12 7v10"
	/>
</svg>
