<script lang="ts">
	let { children } = $props();
</script>

<main
	class="prose prose-lg mx-auto max-w-4xl p-6 prose-headings:my-2 prose-headings:text-slate-700 prose-p:text-slate-700 prose-li:marker:text-slate-700"
>
	{@render children()}
</main>

<style>
	:global(.prose :not(pre) > code) {
		border: 1px solid rgb(148 163 184);
		border-radius: 0.25rem;
		background: rgb(226 232 240);
		padding: 0.1rem 0.35rem;
		font-weight: 400;
		color: rgb(30 41 59);
	}

	:global(.prose :not(pre) > code::before),
	:global(.prose :not(pre) > code::after) {
		content: none;
	}
</style>
