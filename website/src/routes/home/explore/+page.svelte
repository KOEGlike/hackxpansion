<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Explore | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Explore</h1>
		<p class="text-slate-600">Discover what the HackXPansion community is building.</p>
	</header>

	<section aria-labelledby="explore-status" class="content-box p-5">
		<h2 id="explore-status" class="text-2xl font-bold">Community projects are on the way</h2>
		<p class="mt-2">
			Projects will appear here when the community showcase is available. You can still manage
			<a href={resolve('/home/projects')} class="underline">your projects</a> and use the
			<a href={resolve('/home/docs')} class="underline">documentation</a> while you build.
		</p>
	</section>
</main>
