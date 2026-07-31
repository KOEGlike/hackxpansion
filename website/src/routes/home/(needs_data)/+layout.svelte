<script lang="ts">
	import AccountRequired from '$lib/components/account_required.svelte';

	let { data, children } = $props();
</script>

{#if data.user}
	{@render children()}
{:else}
	<AccountRequired />
{/if}
