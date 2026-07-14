<script lang="ts">
	import ProjectForm from '$lib/components/project_form.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	const emptyProject = {
		title: '',
		type: 'card' as const,
		description: '',
		repoUrl: '',
		demoUrl: '',
		thumbnailUrl: '',
		hackatimeProjects: []
	};
</script>

<svelte:head>
	<title>New project · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex w-full flex-col gap-8 p-0 text-slate-800 min-h-full h-fit">
	{#if form?.message}
		<p class="border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	<section class="border-3 border-slate-700 bg-slate-100/70 p-5 w-full min-h-full h-fit">
		<h2 class="mb-4 text-2xl font-bold">Create project</h2>

		<ProjectForm
			action="?/create"
			submitLabel="Create project"
			cancelPath="/home/projects"
			{form}
			initialValues={emptyProject}
			hackatimeProjects={data.hackatimeProjects}
			hackatimeError={data.hackatimeError}
		/>
	</section>
</main>
