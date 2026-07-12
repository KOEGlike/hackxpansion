<script lang="ts">
	import { resolve } from '$app/paths';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	let selectedType = $state<'card' | 'app'>(formTypeValue('type') ?? 'card');

	function formValue(key: string) {
		if (!form || !('values' in form) || !form.values) return '';

		const value = (form.values as Record<string, unknown>)[key];
		return typeof value === 'string' ? value : '';
	}

	function formTypeValue(key: string): 'card' | 'app' | undefined {
		const value = formValue(key);
		return value === 'app' ? 'app' : value === 'card' ? 'card' : undefined;
	}

	function isHackatimeProjectChecked(name: string) {
		if (!form || !('values' in form) || !form.values) return false;
		const value = form.values['hackatimeProjects'] as unknown;
		if (Array.isArray(value)) return (value as string[]).includes(name);
		if (typeof value === 'string' && value.length > 0) {
			return value
				.split(',')
				.map((v) => v.trim())
				.includes(name);
		}
		return false;
	}

	function formatResistor(ohms: number | null): string {
		if (ohms == null) return '—';
		if (ohms >= 1000) {
			const kilo = ohms / 1000;
			return `${kilo}k`;
		}
		return `${ohms}`;
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
			href={resolve('/projects/new')}
			class="rounded-md bg-slate-800 px-4 py-2 text-center text-white hover:bg-slate-700"
		>
			New project
		</a>
	</header>

	{#if form?.message}
		<p class="rounded-md border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	<section class="flex flex-col gap-4">
		<h2 class="text-2xl font-bold">Your projects</h2>

		{#if data.projects.length === 0}
			<p class="rounded-lg border border-slate-500 bg-white/40 p-4">No projects yet.</p>
		{:else}
			{#each data.projects as project (project.id)}
				<article class="rounded-lg border-2 border-slate-700 bg-white/50 p-5">
					<div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
						<div class="flex gap-4">
							{#if project.thumbnailUrl}
								<img
									src={project.thumbnailUrl}
									alt=""
									class="h-20 w-20 rounded-md border border-slate-400 object-cover"
								/>
							{/if}

							<div>
								<h3 class="text-xl font-bold">{project.title}</h3>
								<div class="mt-1 flex items-center gap-2">
									<ProjectStatusBadge status={project.status} />
								</div>
								{#if project.description}
									<p class="mt-2 max-w-2xl">{project.description}</p>
								{/if}
							</div>
						</div>

						<div class="flex items-center gap-2">
							<a
								href={resolve(`/projects/${project.id}/edit`)}
								class="rounded-md border border-slate-700 px-4 py-2 text-sm hover:bg-slate-100"
							>
								Edit
							</a>
							{#if project.status === 'waiting_design' || project.status === 'waiting_build'}
								<form method="post" action="?/withdraw">
									<input type="hidden" name="projectId" value={project.id} />
									<button
										class="rounded-md border border-red-700 px-4 py-2 text-sm text-red-700 hover:bg-red-50"
									>
										Withdraw
									</button>
								</form>
							{:else}
								<form method="post" action="?/submit">
									<input type="hidden" name="projectId" value={project.id} />
									<button
										class="rounded-md bg-blue-700 px-4 py-2 text-sm text-white hover:bg-blue-600 disabled:cursor-not-allowed disabled:bg-slate-400"
										disabled={!project.readiness.canSubmit}
									>
										Submit {project.readiness.phase ?? 'to'} review
									</button>
								</form>
							{/if}
						</div>
					</div>

					<div class="mt-4 grid gap-2 text-sm md:grid-cols-2">
						{#if project.repoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.repoUrl} target="_blank" rel="noreferrer">Repo</a>
						{/if}
						{#if project.demoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.demoUrl} target="_blank" rel="noreferrer">Demo</a>
						{/if}
						<p>Hackatime: {project.hackatimeProjects?.join(', ') || 'none'}</p>
						{#if project.type === 'card'}
							<p>
								Module ID resistors: MD1 = {formatResistor(project.md1)}, MD2 =
								{formatResistor(project.md2)}
							</p>
						{/if}
					</div>

					{#if project.type === 'app'}
						<div class="mt-4 rounded-md bg-blue-50 p-3 text-sm text-blue-950">
							<p class="font-bold">Required resources:</p>
							{#if project.requirements}
								<p>{project.requirements}</p>
							{:else}
								<p>No required resources described.</p>
							{/if}
						</div>
					{/if}

					{#if !project.readiness.canSubmit}
						<div class="mt-4 rounded-md bg-amber-100 p-3 text-sm text-amber-950">
							<p class="font-bold">Before submitting:</p>
							<ul class="list-disc pl-5">
								{#each project.readiness.changes as change (change.field)}
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
