<script lang="ts">
	import ProjectForm from '$lib/components/project_form.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();
</script>

<svelte:head>
	<title>Edit {data.project.title} · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Edit project</h1>
		<div class="mt-1 flex items-center gap-2">
			<span class="text-slate-600">{data.project.title}</span>
			<ProjectStatusBadge status={data.project.status} />
		</div>
	</header>

	{#if form?.message}
		<p class="border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	{#if !data.canEdit}
		<div class="border border-amber-500 bg-amber-100 p-3 text-sm text-amber-950">
			This project can't be edited while it's waiting for Ari review.
		</div>
	{/if}

	<section class="border-2 border-slate-700 bg-slate-100/70 p-5">
		<h2 class="mb-4 text-2xl font-bold">Project details</h2>

		<ProjectForm
			action="?/edit"
			submitLabel="Save changes"
			cancelPath="/home/projects"
			cancelLabel="Back to projects"
			{form}
			initialValues={data.project}
			hackatimeProjects={data.hackatimeProjects}
			hackatimeError={data.hackatimeError}
			disabled={!data.canEdit}
		/>
	</section>
</main>
