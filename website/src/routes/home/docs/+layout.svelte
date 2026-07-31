<script lang="ts">
	let { children } = $props();
</script>

<main
	class="prose prose-lg mx-auto max-w-4xl p-6 prose-headings:my-2 prose-headings:text-slate-700 prose-p:text-slate-700 prose-li:marker:text-slate-700"
>
	{@render children()}
</main>
