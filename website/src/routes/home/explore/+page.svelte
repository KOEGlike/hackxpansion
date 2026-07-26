<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import DropdownSelect from '$lib/components/dropdown_select.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import { getPublicProjectKey, type ExploreFilter } from '$lib/projects/explore';
	import { formatResistor } from '$lib/projects/resistors';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();

	const filterOptions: Array<{ value: ExploreFilter; label: string }> = [
		{ value: 'all', label: 'All projects' },
		{ value: 'not_submitted', label: 'Not submitted' },
		{ value: 'approved_design', label: 'Approved designs' },
		{ value: 'approved_build', label: 'Approved builds' }
	];

	function publicProjectHref(project: PageServerData['projects'][number]) {
		return resolve(`/explore/${getPublicProjectKey(project)}`);
	}
</script>

<svelte:head>
	<title>Explore | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Explore</h1>
		<p class="text-slate-600">Discover what the HackXPansion community is building.</p>
	</header>

	<form
		method="get"
		action={resolve('/home/explore')}
		class="content-box grid gap-3 p-4 sm:grid-cols-[1fr_auto_auto] sm:items-end"
	>
		<label class="flex flex-col gap-1 text-sm font-semibold" for="project-search">
			Search by project name
			<input
				id="project-search"
				name="q"
				type="search"
				value={data.filters.query}
				placeholder="Search projects"
				class="border border-slate-700 bg-white/80 px-3 py-2 font-normal"
			/>
		</label>
		<DropdownSelect
			name="status"
			label="Status"
			options={filterOptions}
			value={data.filters.status}
		/>
		<div class="flex gap-2">
			<button class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700">Filter</button>
			<a
				href={resolve('/home/explore')}
				class="border border-slate-700 px-4 py-2 hover:bg-slate-100"
			>
				Clear
			</a>
		</div>
	</form>

	<section aria-labelledby="project-feed" class="flex flex-col gap-4">
		<div class="flex items-baseline justify-between gap-4">
			<h2 id="project-feed" class="text-2xl font-bold">Project feed</h2>
			<p class="text-sm text-slate-600">
				{data.projects.length} project{data.projects.length === 1 ? '' : 's'}
			</p>
		</div>

		{#if data.projects.length === 0}
			<p class="content-box p-5">No projects match these filters.</p>
		{:else}
			<div class="grid gap-4 md:grid-cols-2">
				{#each data.projects as project (project.id)}
					<article class="content-box flex flex-col gap-4 p-5">
						<div class="flex gap-4">
							{#if project.thumbnailUrl}
								<img
									src={project.thumbnailUrl}
									alt=""
									class="size-20 shrink-0 border border-slate-400 object-cover"
								/>
							{/if}
							<div class="min-w-0">
								<h3 class="text-xl font-bold">
									<a href={publicProjectHref(project)} class="hover:underline">
										{project.title}
									</a>
								</h3>
								<p class="text-sm text-slate-600">by {project.makerName}</p>
								<div class="mt-2 flex flex-wrap items-center gap-2">
									<ProjectStatusBadge status={project.status} />
									<span class="bg-slate-200 px-2 py-0.5 text-xs font-medium uppercase">
										{project.type}{project.tier ? ` · ${project.tier}` : ''}
									</span>
								</div>
							</div>
						</div>

						{#if project.description}
							<p class="line-clamp-3 text-sm text-slate-700">{project.description}</p>
						{/if}

						<div class="mt-auto flex flex-wrap items-center justify-between gap-3 text-sm">
							{#if project.md0 !== null && project.md1 !== null}
								<span class="text-slate-600">
									MD0 {formatResistor(project.md0)} · MD1 {formatResistor(project.md1)}
								</span>
							{:else}
								<span class="text-slate-600">Software app</span>
							{/if}
							<span
								class="flex items-center gap-1 font-semibold"
								aria-label={`${project.currencyPaidOut} currency paid out`}
							>
								<CoinIcon />
								<span aria-hidden="true">{project.currencyPaidOut}</span>
							</span>
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</section>
</main>
