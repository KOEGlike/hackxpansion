<script lang="ts">
	import { asset, resolve } from '$app/paths';

	const announcements = [
		{
			label: 'Hardware update',
			title: 'The console prototype arrived, and it works!',
			description:
				'The first physical Hackxpansion console prototype is here. The display, controls, and app launcher are all up and running.',
			image: asset('/shop/console.png'),
			imageAlt: 'Hackxpansion console prototype showing its app selection screen'
		}
	] as const;
</script>

<svelte:head>
	<title>Home | Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Home</h1>
		<p class="text-slate-600">Build, document, and submit your Hackxpansion projects.</p>
	</header>

	<section aria-labelledby="home-next-steps">
		<h2 id="home-next-steps" class="mb-4 text-2xl font-bold">Get started</h2>
		<div class="grid gap-4 sm:grid-cols-2">
			<a href={resolve('/home/projects')} class="content-box p-5 hover:content-box-hover">
				<h3 class="text-xl font-bold">Your projects</h3>
				<p>Create a card or app, track its progress, and prepare it for review.</p>
			</a>
			<a href={resolve('/home/docs')} class="content-box p-5 hover:content-box-hover">
				<h3 class="text-xl font-bold">Documentation</h3>
				<p>Learn how Hackxpansion works and review the available technical details.</p>
			</a>
		</div>
	</section>

	<section class="flex flex-col gap-4" aria-labelledby="announcements-heading">
		<h2 id="announcements-heading" class="text-2xl font-bold">Announcements</h2>
		{#each announcements as announcement (announcement.title)}
			<article
				class="content-box grid overflow-hidden md:min-h-96 md:grid-cols-[minmax(0,1fr)_20rem]"
			>
				<div class="flex flex-col gap-6 p-6 sm:p-8">
					<p class="text-sm font-bold uppercase tracking-widest text-slate-600">
						{announcement.label} // Online
					</p>
					<div>
						<h3 class="max-w-xl text-4xl leading-none font-bold sm:text-5xl">
							{announcement.title}
						</h3>
						<p class="mt-5 max-w-xl text-lg leading-relaxed text-slate-700">
							{announcement.description}
						</p>
					</div>
				</div>
				<div
					class="flex h-80 items-center justify-center border-t-3 border-slate-700 p-4 md:h-auto md:border-t-0 md:border-l-3"
				>
					<img
						src={announcement.image}
						alt={announcement.imageAlt}
						class="h-full w-full object-contain"
					/>
				</div>
			</article>
		{/each}
	</section>
</main>
