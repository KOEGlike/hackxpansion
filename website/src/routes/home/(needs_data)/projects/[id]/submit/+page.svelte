<script lang="ts">
	import { resolve } from '$app/paths';
	import ReviewSubmissionForm from '$lib/components/review_submission_form.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	function submissionValues() {
		if (!form || form.success || !('values' in form)) return undefined;
		return form.values as Record<string, string> | undefined;
	}
</script>

<svelte:head>
	<title>Submit {data.project.title} · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-3xl flex-col gap-6 p-6 text-slate-800">
	<header>
		<a
			href={resolve(`/home/projects/${data.project.id}`)}
			class="text-sm text-slate-600 hover:underline"
		>
			&larr; Back to {data.project.title}
		</a>
		<h1 class="mt-2 text-4xl font-bold">
			Submit {data.readiness.phase ?? 'project'} review
		</h1>
		<p class="mt-2 text-slate-600">
			Confirm your submission details and share feedback before sending
			<strong>{data.project.title}</strong> to Ari.
		</p>
	</header>

	{#if data.readiness.canSubmit && data.readiness.phase}
		<ReviewSubmissionForm
			action="?/submit"
			phase={data.readiness.phase}
			profile={data.profile}
			repoUrl={data.project.repoUrl}
			errorMessage={form?.message}
			values={submissionValues()}
		/>
	{:else}
		<section class="border border-amber-700 bg-amber-100 p-5 text-amber-950">
			<h2 class="text-xl font-bold">This project is not ready for review</h2>
			<ul class="mt-3 list-disc pl-5">
				{#each data.readiness.changes as change (`${change.field}:${change.message}`)}
					<li>{change.message}</li>
				{/each}
			</ul>
			<a
				href={resolve(`/home/projects/${data.project.id}/edit`)}
				class="mt-4 inline-block border border-slate-800 px-4 py-2 hover:bg-slate-800 hover:text-white"
			>
				Edit project
			</a>
		</section>
	{/if}
</main>
