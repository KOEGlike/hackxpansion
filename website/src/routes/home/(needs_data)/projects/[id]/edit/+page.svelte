<script lang="ts">
	import ProjectForm from '$lib/components/project_form.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();
</script>

<svelte:head>
	<title>Edit {data.project.title} · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex w-full flex-col gap-8 p-0 text-slate-800 min-h-full h-fit">
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

	<section class="p-5 w-full min-h-full h-fit">
		<h2 class="mb-4 text-2xl font-bold">Edit project</h2>
		<div class="mb-4 flex items-center gap-2 text-slate-600">
			<span>{data.project.title}</span>
			<ProjectStatusBadge status={data.project.status} />
		</div>

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
