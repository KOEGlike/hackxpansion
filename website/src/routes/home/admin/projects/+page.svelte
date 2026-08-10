<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import { formatMinutes } from '$lib/projects/domain';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
</script>

<svelte:head>
	<title>Projects | Hackxpansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-6xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin</p>
		<h1 class="text-4xl font-bold">Projects</h1>
		<p class="text-slate-600">View every project, its owner, journals, and review history.</p>
	</header>

	{#if data.projects.length === 0}
		<p class="content-box p-5">No projects have been created.</p>
	{:else}
		<section class="grid gap-4 lg:grid-cols-2" aria-label="All projects">
			{#each data.projects as project (project.id)}
				<article class="content-box flex flex-col gap-4 p-5">
					<div class="flex min-w-0 gap-4">
						{#if project.thumbnailUrl}
							<img
								src={project.thumbnailUrl}
								alt=""
								class="size-20 shrink-0 border border-slate-400 object-cover"
							/>
						{/if}
						<div class="min-w-0">
							<h2 class="truncate text-xl font-bold">
								<a href={resolve(`/home/admin/projects/${project.id}`)} class="hover:underline">
									{project.title}
								</a>
							</h2>
							<p class="truncate text-sm text-slate-600">
								{project.ownerName} · {project.ownerEmail}
							</p>
							<div class="mt-2 flex flex-wrap items-center gap-2">
								<ProjectStatusBadge status={project.status} />
								<span class="bg-slate-200 px-2 py-0.5 text-xs font-medium uppercase">
									{project.type}
								</span>
								{#if project.tier}
									<span class="bg-slate-200 px-2 py-0.5 text-xs font-medium uppercase">
										{project.tier}
									</span>
								{/if}
							</div>
						</div>
					</div>

					{#if project.description}
						<p class="line-clamp-2 text-sm">{project.description}</p>
					{/if}

					<div
						class="mt-auto flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-slate-300 pt-3 text-sm text-slate-600"
					>
						<span>{project.journalCount} journal{project.journalCount === 1 ? '' : 's'}</span>
						<span>{formatMinutes(project.totalJournalMinutes)} journaled</span>
						<span>{project.reviewCount} review{project.reviewCount === 1 ? '' : 's'}</span>
						<span
							class="flex items-center gap-1"
							aria-label={`${project.currencyPaidOut} currency paid out`}
						>
							<CoinIcon />
							<span aria-hidden="true">{project.currencyPaidOut} paid</span>
						</span>
					</div>
				</article>
			{/each}
		</section>
	{/if}
</main>
