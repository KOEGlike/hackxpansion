<script lang="ts">
	import { resolve } from '$app/paths';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import JournalEditor from '$lib/components/journal_editor.svelte';
	import Markdown from '$lib/components/markdown.svelte';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import { formatMinutes, isWaitingForReview } from '$lib/projects/domain';
	import { isValidJournalDuration } from '$lib/projects/journal';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	let durationInput = $state('');
	let textInput = $state('');
	let journalTab: 'write' | 'preview' = $state('write');

	let editingJournalId = $state<string | null>(null);
	let editDuration = $state('');
	let editText = $state('');
	let editTab: 'write' | 'preview' = $state('write');

	function formatDate(date: Date | string): string {
		const d = new Date(date);
		return d.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function reviewEventLabel(event: string): string {
		const labels: Record<string, string> = {
			approved: 'Approved',
			changes: 'Changes requested',
			rejected: 'Rejected',
			reverted: 'Reverted',
			requeued: 'Requeued',
			fraud: 'Fraud detected'
		};
		return labels[event] ?? event;
	}

	function cancelEdit() {
		editingJournalId = null;
	}

	interface TimelineItem {
		type: 'journal' | 'review';
		id?: string;
		date: Date;
		label: string;
		detail: string;
		durationMinutes?: number;
		color: string;
		borderColor: string;
	}

	function reviewColor(event: string): { bg: string; border: string } {
		switch (event) {
			case 'approved':
				return { bg: 'bg-green-100', border: 'border-green-700' };
			case 'changes':
				return { bg: 'bg-amber-100', border: 'border-amber-700' };
			case 'rejected':
				return { bg: 'bg-red-100', border: 'border-red-700' };
			case 'fraud':
				return { bg: 'bg-red-100', border: 'border-red-700' };
			default:
				return { bg: 'bg-blue-100', border: 'border-blue-700' };
		}
	}

	let timeline = $derived.by(() => {
		const items: TimelineItem[] = [];

		for (const j of data.journals) {
			items.push({
				type: 'journal',
				id: j.id,
				date: new Date(j.createdAt),
				label: `Journal entry — ${formatMinutes(j.durationInMinutes)}`,
				detail: j.text,
				durationMinutes: j.durationInMinutes,
				color: 'bg-slate-100',
				borderColor: 'border-slate-700'
			});
		}

		for (const r of data.reviews) {
			const c = reviewColor(r.event);
			items.push({
				type: 'review',
				id: r.id,
				date: new Date(r.receivedAt),
				label: `Review: ${reviewEventLabel(r.event)}`,
				detail: r.noteToMaker ?? 'Review outcome recorded.',
				color: c.bg,
				borderColor: c.border
			});
		}

		items.sort((a, b) => b.date.getTime() - a.date.getTime());
		return items;
	});

	let isWaiting = $derived(isWaitingForReview(data.project.status));
</script>

<svelte:head>
	<title>{data.project.title} · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
		<div>
			<a href={resolve('/home/projects')} class="text-sm text-slate-600 hover:underline">
				&larr; Back to projects
			</a>
			<div class="mt-1 flex items-center gap-3">
				<h1 class="text-4xl font-bold">{data.project.title}</h1>
				<ProjectStatusBadge status={data.project.status} />
			</div>
			{#if data.project.description}
				<p class="mt-1 max-w-2xl text-slate-600">{data.project.description}</p>
			{/if}
			<div class="mt-2 flex gap-3 text-sm">
				<!-- eslint-disable svelte/no-navigation-without-resolve -- validated external URLs -->
				{#if data.project.repoUrl}
					<a
						class="underline"
						href={data.project.repoUrl}
						target="_blank"
						rel="noopener noreferrer"
					>
						Repo
					</a>
				{/if}
				{#if data.project.demoUrl}
					<a
						class="underline"
						href={data.project.demoUrl}
						target="_blank"
						rel="noopener noreferrer"
					>
						Demo
					</a>
				{/if}
				<!-- eslint-enable svelte/no-navigation-without-resolve -->
			</div>
		</div>

		<div class="flex items-start gap-2">
			{#if data.canEdit}
				<a
					href={resolve(`/home/projects/${data.project.id}/edit`)}
					class="border border-slate-800 px-4 py-2 text-sm hover:bg-slate-800 hover:text-white"
				>
					Edit
				</a>
			{/if}

			{#if isWaiting}
				<form method="post" action="?/withdraw">
					<button class="border border-red-700 px-4 py-2 text-sm text-red-700 hover:bg-red-50">
						Withdraw
					</button>
				</form>
			{:else if data.readiness.canSubmit}
				<form method="post" action="?/submit">
					<button class="bg-slate-800 px-4 py-2 text-sm text-white hover:bg-slate-700">
						Submit {data.readiness.phase ?? 'for'} review
					</button>
				</form>
			{/if}
		</div>
	</header>

	{#if form?.message}
		<p
			class="border border-slate-700 bg-white/50 p-3 text-sm"
			class:border-green-700={form.success}
			class:border-red-700={!form.success}
			class:text-green-900={form.success}
			class:text-red-900={!form.success}
		>
			{form.message}
		</p>
	{/if}

	<section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Currency paid</p>
			<p
				class="mt-1 flex items-center gap-1 text-2xl font-bold"
				aria-label={`${data.project.currencyPaidOut} currency paid out`}
			>
				<CoinIcon class="size-6" />
				<span aria-hidden="true">{data.project.currencyPaidOut}</span>
			</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Journal entries</p>
			<p class="mt-1 text-2xl font-bold">{data.stats.journalCount}</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Journaled time</p>
			<p class="mt-1 text-2xl font-bold">
				{formatMinutes(data.stats.totalJournalMinutes)}
			</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Hackatime</p>
			<p class="mt-1 text-2xl font-bold">
				{data.hackatime.error ? 'Unavailable' : formatMinutes(data.hackatime.minutes)}
			</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Total tracked</p>
			<p class="mt-1 text-2xl font-bold">
				{data.hackatime.error
					? formatMinutes(data.stats.totalJournalMinutes)
					: formatMinutes(data.stats.totalJournalMinutes + data.hackatime.minutes)}
			</p>
		</article>
	</section>
	{#if data.hackatime.error}
		<p class="border border-amber-700 bg-amber-100 p-3 text-sm text-amber-950">
			{data.hackatime.error} Totals shown here exclude Hackatime.
		</p>
	{/if}

	{#if !data.readiness.canSubmit && !isWaiting && data.readiness.changes.length > 0}
		<div class="bg-amber-100 p-3 text-sm text-amber-950">
			<p class="font-bold">Before submitting:</p>
			<ul class="list-disc pl-5">
				{#each data.readiness.changes as change (`${change.field}:${change.message}`)}
					<li>{change.message}</li>
				{/each}
			</ul>
		</div>
	{/if}

	<section class="flex flex-col gap-4">
		<h2 class="text-2xl font-bold">Timeline</h2>

		{#if form?.journalSuccess}
			<p class="border border-green-700 bg-green-100 p-3 text-sm text-green-900">
				Journal entry saved.
			</p>
		{/if}
		{#if form?.journalError}
			<p class="border border-red-700 bg-red-100 p-3 text-sm text-red-900">
				{form.journalError}
			</p>
		{/if}

		<div class="flex flex-col gap-4">
			{#if !isWaiting}
				<div class="relative flex items-start gap-4">
					{#if timeline.length > 0}
						<div
							class="absolute left-5 top-10 w-0.5 bg-slate-400"
							style="height: calc(100% + 1rem)"
						></div>
					{/if}
					<div
						class="relative z-10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border-2 border-dashed border-slate-500 bg-white/50 text-xs font-bold text-slate-500"
					>
						J
					</div>
					<div class="flex-1">
						<p class="mb-2 text-sm font-semibold text-slate-600">New journal entry</p>
						<form method="post" action="?/createJournal" class="flex flex-col gap-3">
							<JournalEditor
								idPrefix="new"
								bind:duration={durationInput}
								bind:text={textInput}
								bind:tab={journalTab}
							/>
							<div class="flex justify-end">
								<button
									type="submit"
									class="bg-slate-800 px-4 py-2 text-sm text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
									disabled={!isValidJournalDuration(durationInput) || !textInput.trim()}
								>
									Log entry
								</button>
							</div>
						</form>
					</div>
				</div>
			{/if}

			{#if timeline.length === 0}
				<p class="border border-slate-500 bg-white/40 p-4 text-sm">
					No journal entries or reviews yet.
				</p>
			{:else}
				{#each timeline as item, i (`${item.type}:${item.id}`)}
					<div class="relative flex items-start gap-4">
						{#if i < timeline.length - 1}
							<div
								class="absolute left-5 top-10 w-0.5 bg-slate-400"
								style="height: calc(100% + 1rem)"
							></div>
						{/if}
						<div
							class="relative z-10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full border-2 text-xs font-bold {item.color} {item.borderColor}"
						>
							{#if item.type === 'journal'}
								J
							{:else}
								R
							{/if}
						</div>
						<div class="flex-1 pt-1.5">
							<div class="flex items-baseline justify-between gap-2">
								<p class="font-semibold">{item.label}</p>
								<p class="text-xs text-slate-500">{formatDate(item.date)}</p>
							</div>
							{#if item.type === 'journal' && item.id && editingJournalId === item.id}
								<form method="post" action="?/editJournal" class="mt-2 flex flex-col gap-3">
									<input type="hidden" name="journalId" value={item.id} />
									<JournalEditor
										idPrefix={`edit-${item.id}`}
										bind:duration={editDuration}
										bind:text={editText}
										bind:tab={editTab}
									/>
									<div class="flex justify-end gap-2">
										<button
											type="button"
											class="border border-slate-700 px-3 py-1.5 text-sm hover:bg-slate-200"
											onclick={cancelEdit}
										>
											Cancel
										</button>
										<button
											type="submit"
											class="bg-slate-800 px-3 py-1.5 text-sm text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
											disabled={!isValidJournalDuration(editDuration) || !editText.trim()}
										>
											Save
										</button>
									</div>
								</form>
							{:else}
								{#if item.type === 'journal'}
									<div class="mt-1"><Markdown text={item.detail} /></div>
								{:else}
									<p class="mt-1 text-sm text-slate-600">{item.detail}</p>
								{/if}
								{#if item.type === 'journal' && item.id && !isWaiting}
									<button
										type="button"
										class="mt-1 text-xs text-slate-500 hover:underline"
										onclick={() => {
											editingJournalId = item.id ?? null;
											editDuration = String(item.durationMinutes ?? '');
											editText = item.detail;
											editTab = 'write';
										}}
									>
										Edit
									</button>
								{/if}
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</section>
</main>
