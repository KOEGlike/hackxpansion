<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Home | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Home</h1>
		<p class="text-slate-600">Build, document, and submit your HackXPansion projects.</p>
	</header>

	<section aria-labelledby="home-next-steps">
		<h2 id="home-next-steps" class="mb-4 text-2xl font-bold">Get started</h2>
		<div class="grid gap-4 sm:grid-cols-2">
			<a href={resolve('/home/projects')} class="content-box p-5 hover:content-box-hover">
				<h3 class="text-xl font-bold">Your projects</h3>
				<p>Create a card or app, track its progress, and prepare it for review.</p>
			</a>
			<a href={resolve('/docs')} class="content-box p-5 hover:content-box-hover">
				<h3 class="text-xl font-bold">Documentation</h3>
				<p>Learn how HackXPansion works and review the available technical details.</p>
			</a>
		</div>
	</section>
</main>
