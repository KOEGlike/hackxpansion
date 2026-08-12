<script lang="ts">
	import { resolve } from '$app/paths';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();
	let deleteDialog: HTMLDialogElement;
	let selectedSubmissionId = $state<string | null>(null);
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });
	const selectedSubmission = $derived(
		data.submissions.find((submission) => submission.id === selectedSubmissionId)
	);

	$effect(() => {
		if (form?.success === false && form.submissionId) {
			selectedSubmissionId = form.submissionId;
			if (deleteDialog && !deleteDialog.open) deleteDialog.showModal();
		}
	});

	function confirmDelete(submissionId: string) {
		selectedSubmissionId = submissionId;
		deleteDialog.showModal();
	}
</script>

<svelte:head>
	<title>Submissions | Hackxpansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-7xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin</p>
		<h1 class="text-4xl font-bold">Project submissions</h1>
		<p class="text-slate-600">Review every project submission snapshot sent to ARI.</p>
	</header>

	{#if form?.message}
		<p
			class="border p-3 text-sm"
			class:border-green-700={form.success}
			class:bg-green-100={form.success}
			class:border-red-700={!form.success}
			class:bg-red-100={!form.success}
		>
			{form.message}
		</p>
	{/if}

	{#if data.submissions.length === 0}
		<p class="content-box p-5">No project submissions have been created.</p>
	{:else}
		<div class="overflow-x-auto border border-slate-500 bg-white/40">
			<table class="w-full min-w-5xl border-collapse text-left text-sm">
				<thead class="bg-slate-200 text-xs uppercase tracking-wide text-slate-600">
					<tr>
						<th class="px-4 py-3">Submitted by</th>
						<th class="px-4 py-3">Project</th>
						<th class="px-4 py-3">Phase</th>
						<th class="px-4 py-3">Submitted</th>
						<th class="px-4 py-3">NPS</th>
						<th class="px-4 py-3">ARI external ID</th>
						<th class="px-4 py-3 text-right">Action</th>
					</tr>
				</thead>
				<tbody>
					{#each data.submissions as submission (submission.id)}
						<tr class="border-t border-slate-300 align-middle">
							<td class="px-4 py-3">
								<div class="flex min-w-48 items-center gap-3">
									{#if submission.userImage}
										<img
											src={submission.userImage}
											alt=""
											class="size-10 shrink-0 border border-slate-400 object-cover"
										/>
									{:else}
										<div
											class="flex size-10 shrink-0 items-center justify-center border border-slate-400 bg-slate-200 font-bold"
											aria-hidden="true"
										>
											{submission.userName.slice(0, 1).toUpperCase()}
										</div>
									{/if}
									<div class="min-w-0">
										<p class="truncate font-bold">{submission.userName}</p>
										<p class="truncate text-xs text-slate-500">{submission.userEmail}</p>
									</div>
								</div>
							</td>
							<td class="px-4 py-3">
								<a
									href={resolve(`/home/admin/projects/${submission.projectId}`)}
									class="font-bold hover:underline"
								>
									{submission.projectTitle}
								</a>
								<p class="mt-1 text-xs uppercase text-slate-500">{submission.projectStatus}</p>
							</td>
							<td class="px-4 py-3 capitalize">{submission.phase}</td>
							<td class="whitespace-nowrap px-4 py-3">
								{dateFormatter.format(new Date(submission.createdAt))}
							</td>
							<td class="px-4 py-3">{submission.nps}/10</td>
							<td class="max-w-72 px-4 py-3 font-mono text-xs break-all">
								{submission.ariExternalId}
							</td>
							<td class="px-4 py-3 text-right">
								<button
									type="button"
									class="border border-red-800 bg-red-800 px-3 py-1.5 font-bold text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:border-red-300 disabled:bg-red-300"
									disabled={submission.isWaitingForReview}
									title={submission.isWaitingForReview
										? 'This submission is waiting for ARI review.'
										: 'Delete submission'}
									onclick={() => confirmDelete(submission.id)}
								>
									Delete
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</main>

<dialog
	bind:this={deleteDialog}
	class="m-auto w-[min(32rem,calc(100%-2rem))] border border-slate-900 bg-white p-0 text-slate-900 backdrop:bg-slate-950/60"
	aria-labelledby="delete-submission-title"
>
	<div class="p-6">
		<h2 id="delete-submission-title" class="text-2xl font-bold">Are you sure?</h2>
		<p class="mt-3">
			Deleting the {selectedSubmission?.phase ?? ''} submission for
			<strong>{selectedSubmission?.projectTitle ?? 'this project'}</strong> is permanent. Existing review
			records and paid currency will not be changed.
		</p>
		{#if form?.success === false && form.submissionId === selectedSubmissionId}
			<p class="mt-4 border border-red-700 bg-red-100 p-3 text-sm" role="alert">
				{form.message}
			</p>
		{/if}
		<div class="mt-6 flex justify-end gap-3">
			<form method="dialog">
				<button class="border border-slate-800 px-4 py-2 font-bold hover:bg-slate-100"
					>Cancel</button
				>
			</form>
			<form method="POST" action="?/delete">
				<input type="hidden" name="submissionId" value={selectedSubmissionId ?? ''} />
				<button
					class="border border-red-800 bg-red-800 px-4 py-2 font-bold text-white hover:bg-red-700"
				>
					Delete
				</button>
			</form>
		</div>
	</div>
</dialog>
