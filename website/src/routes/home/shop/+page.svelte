<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Shop | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Shop</h1>
		<p class="text-slate-600">HackXPansion rewards and ordering information.</p>
	</header>

	<section aria-labelledby="shop-status" class="content-box p-5">
		<h2 id="shop-status" class="text-2xl font-bold">The shop is not open yet</h2>
		<p class="mt-2">
			Ordering details will appear here when they are available. In the meantime, continue working
			on <a href={resolve('/home/projects')} class="underline">your projects</a> or review the
			<a href={resolve('/docs')} class="underline">documentation</a>.
		</p>
	</section>
</main>
