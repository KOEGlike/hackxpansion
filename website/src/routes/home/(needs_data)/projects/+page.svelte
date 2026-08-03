<script lang="ts">
	import { resolve } from '$app/paths';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import YswsEligibilityNotice from '$lib/components/ysws_eligibility_notice.svelte';
	import { formatMinutes, isWaitingForReview } from '$lib/projects/domain';
	import { formatResistor as formatResistorValue } from '$lib/projects/resistors';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	function formatResistor(ohms: number | null): string {
		if (ohms == null) return '—';
		return formatResistorValue(ohms);
	}
</script>

<svelte:head>
	<title>Projects · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
		<div>
			<h1 class="text-4xl font-bold">Projects</h1>
			<p class="text-slate-600">Create a project, fill in the basics, then submit it to Ari.</p>
		</div>
		<a
			href={resolve('/home/projects/new')}
			class=" bg-slate-800 px-4 py-2 text-center text-white hover:bg-slate-700"
		>
			New project
		</a>
	</header>

	{#if form?.message}
		<p class=" border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}
	{#if data.hackatimeError}
		<p class="border border-amber-700 bg-amber-100 p-3 text-sm text-amber-950">
			{data.hackatimeError} Project totals currently include journals only.
		</p>
	{/if}
	{#if !data.yswsEligible}
		<YswsEligibilityNotice />
	{/if}

	<section class="flex flex-col gap-4">
		<h2 class="text-2xl font-bold">Your projects</h2>

		{#if data.projects.length === 0}
			<p class=" border border-slate-500 bg-white/40 p-4">No projects yet.</p>
		{:else}
			{#each data.projects as project (project.id)}
				<article class="content-box p-5">
					<div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
						<div class="flex gap-4">
							{#if project.thumbnailUrl}
								<img
									src={project.thumbnailUrl}
									alt=""
									class="h-20 w-20 border border-slate-400 object-cover"
								/>
							{/if}

							<div>
								<h3 class="text-xl font-bold">
									<a href={resolve(`/home/projects/${project.id}`)} class="hover:underline"
										>{project.title}</a
									>
								</h3>
								<div class="mt-1 flex items-center gap-2">
									<ProjectStatusBadge status={project.status} />
									{#if project.tier}
										<span class=" bg-slate-200 px-2 py-0.5 text-xs font-medium uppercase"
											>{project.tier}</span
										>
									{/if}
								</div>
								{#if project.description}
									<p class="mt-2 max-w-2xl">{project.description}</p>
								{/if}
							</div>
						</div>

						<div class="flex items-center gap-2">
							<a
								href={resolve(`/home/projects/${project.id}/edit`)}
								class="border border-slate-800 px-4 py-2 text-sm hover:bg-slate-800 hover:text-white"
							>
								Edit
							</a>
							{#if isWaitingForReview(project.status)}
								<form method="post" action="?/withdraw">
									<input type="hidden" name="projectId" value={project.id} />
									<button
										class=" border border-red-700 px-4 py-2 text-sm text-red-700 hover:bg-red-50"
									>
										Withdraw
									</button>
								</form>
							{:else}
								<form method="post" action="?/submit">
									<input type="hidden" name="projectId" value={project.id} />
									<button
										class=" bg-slate-800 px-4 py-2 text-sm text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
										disabled={!project.readiness.canSubmit}
									>
										Submit {project.readiness.phase ?? 'to'} review
									</button>
								</form>
							{/if}
						</div>
					</div>

					<div class="mt-4 flex flex-row gap-4">
						{#if project.repoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.repoUrl} target="_blank" rel="noreferrer">Repo</a>
							|
						{/if}
						{#if project.demoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.demoUrl} target="_blank" rel="noreferrer">Demo</a>
							|
						{/if}
						{#if project.type === 'card'}
							<p>
								MD0:{formatResistor(project.md0)} MD1:{formatResistor(project.md1)}
							</p>
							|
						{/if}
						<p>Hackatime:{project.hackatimeProjects?.join(',') || 'none'}</p>
					</div>

					<div class="mt-4 flex flex-wrap gap-4 text-sm text-slate-600">
						<span>{project.journalCount} journal{project.journalCount === 1 ? '' : 's'}</span>
						|
						<span class="font-semibold text-slate-700"
							>{formatMinutes(project.totalTrackedMinutes)} total tracked</span
						>
						|
						<span>{project.reviewCount} review{project.reviewCount === 1 ? '' : 's'}</span>
						|
						<span
							class="flex items-center gap-1 font-semibold text-slate-700"
							aria-label={`${project.currencyPaidOut} currency paid out`}
						>
							<CoinIcon />
							<span aria-hidden="true">{project.currencyPaidOut} paid</span>
						</span>
					</div>

					{#if !project.readiness.canSubmit}
						<div class="mt-4 bg-amber-100 p-3 text-sm text-amber-950">
							<p class="font-bold">Before submitting:</p>
							<ul class="list-disc pl-5">
								{#each project.readiness.changes as change (`${change.field}:${change.message}`)}
									<li>{change.message}</li>
								{/each}
							</ul>
						</div>
					{/if}
				</article>
			{/each}
		{/if}
	</section>
</main>
