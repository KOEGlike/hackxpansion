<script lang="ts">
	import { resolve } from '$app/paths';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	function formValue(key: string) {
		if (form && 'values' in form && form.values) {
			const value = form.values[key];
			if (typeof value === 'string') return value;
		}

		return initial(key);
	}

	function formValueList(key: string): string[] {
		if (form && 'values' in form && form.values) {
			const value = form.values[key];
			if (Array.isArray(value)) return value;
			if (typeof value === 'string' && value.trim()) return [value];
		}

		return data.project.hackatimeProjects ?? [];
	}

	function isChecked(name: string) {
		return formValueList('hackatimeProjects').includes(name);
	}

	function formatDuration(totalSeconds: number) {
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		if (hours > 0) return `${hours}h ${minutes}m`;
		return `${minutes}m`;
	}

	function initial(key: string): string {
		switch (key) {
			case 'title':
				return data.project.title;
			case 'description':
				return data.project.description ?? '';
			case 'repoUrl':
				return data.project.repoUrl ?? '';
			case 'demoUrl':
				return data.project.demoUrl ?? '';
			case 'thumbnailUrl':
				return data.project.thumbnailUrl ?? '';
			default:
				return '';
		}
	}
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
		<p class="rounded-md border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	{#if !data.canEdit}
		<div class="rounded-md border border-amber-500 bg-amber-100 p-3 text-sm text-amber-950">
			This project can't be edited while it's waiting for Ari review.
		</div>
	{/if}

	<section class="rounded-lg border-2 border-slate-700 bg-slate-100/70 p-5">
		<h2 class="mb-4 text-2xl font-bold">Project details</h2>

		<form method="post" action="?/edit" class="grid gap-4 md:grid-cols-2">
			<label class="flex flex-col gap-1">
				<span>Title *</span>
				<input
					name="title"
					required
					value={formValue('title')}
					disabled={!data.canEdit}
					class="rounded-md"
				/>
			</label>

			<div class="flex flex-col gap-1">
				<span>Hackatime projects</span>
				<p class="text-xs text-slate-500">Required before submitting to Ari.</p>
			</div>

			<div class="md:col-span-2">
				{#if data.hackatimeError}
					<p class="text-sm text-amber-700">{data.hackatimeError}</p>
				{/if}
				{#if data.hackatimeProjects.length === 0}
					<p class="text-sm text-slate-500">
						No Hackatime projects found. Make sure you have heartbeats logged in Hackatime.
					</p>
				{:else}
					<div
						class="grid max-h-60 overflow-y-auto gap-1 rounded-md border border-slate-300 p-2 md:grid-cols-2"
					>
						{#each data.hackatimeProjects as project (project.name)}
							<label class="flex items-center gap-2 p-1 text-sm hover:bg-slate-100">
								<input
									type="checkbox"
									name="hackatimeProjects"
									value={project.name}
									checked={isChecked(project.name)}
									disabled={!data.canEdit}
									class="rounded"
								/>
								<span>{project.name}</span>
								<span class="text-slate-400">{formatDuration(project.totalSeconds)}</span>
							</label>
						{/each}
					</div>
				{/if}
			</div>

			<label class="flex flex-col gap-1 md:col-span-2">
				<span>Description</span>
				<textarea name="description" rows="3" disabled={!data.canEdit} class="rounded-md"
					>{formValue('description')}</textarea
				>
			</label>

			<label class="flex flex-col gap-1">
				<span>Repo URL</span>
				<input
					name="repoUrl"
					value={formValue('repoUrl')}
					disabled={!data.canEdit}
					class="rounded-md"
				/>
			</label>

			<label class="flex flex-col gap-1">
				<span>Thumbnail URL</span>
				<input
					name="thumbnailUrl"
					value={formValue('thumbnailUrl')}
					disabled={!data.canEdit}
					class="rounded-md"
				/>
			</label>

			<label class="flex flex-col gap-1 md:col-span-2">
				<span>Demo URL</span>
				<input
					name="demoUrl"
					placeholder="Required before build review"
					value={formValue('demoUrl')}
					disabled={!data.canEdit}
					class="rounded-md"
				/>
			</label>

			<div class="flex items-center gap-4 md:col-span-2">
				<button
					class="rounded-md bg-slate-800 px-4 py-2 text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
					disabled={!data.canEdit}
				>
					Save changes
				</button>
				<a href={resolve('/projects')} class="text-sm text-slate-600 hover:underline"
					>Back to projects</a
				>
			</div>
		</form>
	</section>
</main>
