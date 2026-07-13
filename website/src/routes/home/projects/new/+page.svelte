<script lang="ts">
	import ProjectForm from '$lib/components/project_form.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	const emptyProject = {
		title: '',
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

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">New project</h1>
		<p class="text-slate-600">Fill in the basics, then submit it to Ari.</p>
	</header>

	{#if form?.message}
		<p class="border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	<section class="border-2 border-slate-700 bg-slate-100/70 p-5">
		<h2 class="mb-4 text-2xl font-bold">Create project</h2>

		<ProjectForm
			action="?/create"
			submitLabel="Create project"
			cancelPath="/projects"
			{form}
			initialValues={emptyProject}
			hackatimeProjects={data.hackatimeProjects}
			hackatimeError={data.hackatimeError}
		/>
	</section>
</main>
