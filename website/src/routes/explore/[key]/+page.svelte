<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import GridBg from '$lib/components/grid_bg.svelte';
	import Markdown from '$lib/components/markdown.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import { formatMinutes } from '$lib/projects/domain';
	import { formatResistor } from '$lib/projects/resistors';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });
</script>

<svelte:head>
	<title>{data.project.title} | Hackxpansion</title>
	<meta
		name="description"
		content={data.project.description ?? `A project by ${data.project.makerName}`}
	/>
</svelte:head>

<div class="relative min-h-screen text-slate-800">
	<GridBg />
	<main class="relative z-10 mx-auto flex max-w-4xl flex-col gap-6 p-6">
		<a href={resolve('/home/explore')} class="w-fit text-sm text-slate-600 hover:underline">
			&larr; Back to Explore
		</a>

		<article class="content-box overflow-hidden">
			{#if data.project.thumbnailUrl}
				<img
					src={data.project.thumbnailUrl}
					alt=""
					class="max-h-96 w-full border-b border-slate-500 object-cover"
				/>
			{/if}

			<div class="flex flex-col gap-6 p-6">
				<header class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
					<div>
						<h1 class="text-4xl font-bold">{data.project.title}</h1>
						<div class="mt-2 flex flex-wrap items-center gap-2">
							<ProjectStatusBadge status={data.project.status} />
							<span class="bg-slate-200 px-2 py-0.5 text-xs font-medium uppercase">
								{data.project.type}{data.project.tier ? ` · ${data.project.tier}` : ''}
							</span>
						</div>
					</div>

					<div
						class="flex items-center gap-1 text-xl font-bold"
						aria-label={`${data.project.currencyPaidOut} currency paid out`}
					>
						<CoinIcon class="size-6" />
						<span aria-hidden="true">{data.project.currencyPaidOut}</span>
					</div>
				</header>

				<section aria-label="Maker" class="flex items-center gap-3 border-y border-slate-300 py-3">
					{#if data.project.makerImage}
						<img src={data.project.makerImage} alt="" class="size-10 object-cover" />
					{/if}
					<p>
						<span class="text-sm text-slate-600">Made by</span><br />
						<span class="font-semibold">{data.project.makerName}</span>
					</p>
				</section>

				{#if data.project.description}
					<section aria-labelledby="project-description">
						<h2 id="project-description" class="text-xl font-bold">About this project</h2>
						<p class="mt-2 whitespace-pre-line">{data.project.description}</p>
					</section>
				{/if}

				<dl class="grid gap-4 sm:grid-cols-2">
					{#if data.project.md0 !== null && data.project.md1 !== null}
						<div class="border border-slate-400 bg-white/50 p-3">
							<dt class="text-xs uppercase tracking-wide text-slate-600">MD0</dt>
							<dd class="text-xl font-bold">{formatResistor(data.project.md0)}</dd>
						</div>
						<div class="border border-slate-400 bg-white/50 p-3">
							<dt class="text-xs uppercase tracking-wide text-slate-600">MD1</dt>
							<dd class="text-xl font-bold">{formatResistor(data.project.md1)}</dd>
						</div>
					{/if}
				</dl>

				<div class="flex flex-wrap gap-3">
					<!-- eslint-disable svelte/no-navigation-without-resolve -- validated external URLs -->
					{#if data.project.repoUrl}
						<a
							href={data.project.repoUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700"
						>
							View repository
						</a>
					{/if}
					{#if data.project.demoUrl}
						<a
							href={data.project.demoUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="border border-slate-800 px-4 py-2 hover:bg-slate-100"
						>
							Open demo
						</a>
					{/if}
					<!-- eslint-enable svelte/no-navigation-without-resolve -->
				</div>
			</div>
		</article>

		<section class="flex flex-col gap-4" aria-labelledby="project-journals">
			<div>
				<h2 id="project-journals" class="text-2xl font-bold">Journals</h2>
				<p class="text-sm text-slate-600">Follow the progress behind this project.</p>
			</div>

			{#if data.journals.length === 0}
				<p class="content-box p-5">No journal entries have been published yet.</p>
			{:else}
				{#each data.journals as entry (entry.id)}
					<article class="content-box p-5">
						<header class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
							<h3 class="font-bold">{formatMinutes(entry.durationInMinutes)}</h3>
							<time
								datetime={new Date(entry.createdAt).toISOString()}
								class="text-xs text-slate-500"
							>
								{dateFormatter.format(new Date(entry.createdAt))}
							</time>
						</header>
						<Markdown text={entry.text} />
					</article>
				{/each}
			{/if}
		</section>
	</main>
</div>
