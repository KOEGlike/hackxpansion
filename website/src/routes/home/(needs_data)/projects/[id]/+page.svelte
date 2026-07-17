<script lang="ts">
	import { resolve } from '$app/paths';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	let durationInput = $state('');
	let textInput = $state('');

	function formatMinutes(minutes: number): string {
		const h = Math.floor(minutes / 60);
		const m = minutes % 60;
		if (h === 0) return `${m}m`;
		if (m === 0) return `${h}h`;
		return `${h}h ${m}m`;
	}

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

	interface TimelineItem {
		type: 'journal' | 'review';
		date: Date;
		label: string;
		detail: string;
		color: string;
		borderColor: string;
	}

	function reviewColor(event: string): { bg: string; border: string; detailPrefix: string } {
		switch (event) {
			case 'approved':
				return { bg: 'bg-green-100', border: 'border-green-700', detailPrefix: 'approved' };
			case 'changes':
				return {
					bg: 'bg-amber-100',
					border: 'border-amber-700',
					detailPrefix: 'changes requested'
				};
			case 'rejected':
				return { bg: 'bg-red-100', border: 'border-red-700', detailPrefix: 'rejected' };
			case 'fraud':
				return { bg: 'bg-red-100', border: 'border-red-700', detailPrefix: 'fraud detected' };
			default:
				return { bg: 'bg-blue-100', border: 'border-blue-700', detailPrefix: 'outcome' };
		}
	}

	let timeline = $derived.by(() => {
		const items: TimelineItem[] = [];

		for (const j of data.journals) {
			items.push({
				type: 'journal',
				date: new Date(j.createdAt),
				label: 'Journal entry',
				detail: `${formatMinutes(j.durationInMinutes)} — ${j.text}`,
				color: 'bg-slate-100',
				borderColor: 'border-slate-700'
			});
		}

		for (const r of data.reviews) {
			const c = reviewColor(r.event);
			const detail = r.noteToMaker
				? `${formatMinutes(r.approvedMinutes ?? 0)} ${c.detailPrefix} — ${r.noteToMaker}`
				: `${formatMinutes(r.approvedMinutes ?? 0)} ${c.detailPrefix}`;
			items.push({
				type: 'review',
				date: new Date(r.receivedAt),
				label: `Review: ${reviewEventLabel(r.event)}`,
				detail,
				color: c.bg,
				borderColor: c.border
			});
		}

		items.sort((a, b) => b.date.getTime() - a.date.getTime());
		return items;
	});

	let isWaiting = $derived(
		data.project.status === 'waiting_design' || data.project.status === 'waiting_build'
	);
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
				{#if data.project.repoUrl}
					<a class="underline" href={data.project.repoUrl} target="_blank" rel="external"> Repo </a>
				{/if}
				{#if data.project.demoUrl}
					<a class="underline" href={data.project.demoUrl} target="_blank" rel="external"> Demo </a>
				{/if}
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

	<section class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
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
				{formatMinutes(data.hackatimeMinutes)}
			</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Approved time</p>
			<p class="mt-1 text-2xl font-bold">
				{formatMinutes(data.stats.totalApprovedMinutes)}
			</p>
		</article>
	</section>

	{#if !data.readiness.canSubmit && !isWaiting && data.readiness.changes.length > 0}
		<div class="bg-amber-100 p-3 text-sm text-amber-950">
			<p class="font-bold">Before submitting:</p>
			<ul class="list-disc pl-5">
				{#each data.readiness.changes as change (change.field)}
					<li>{change.message}</li>
				{/each}
			</ul>
		</div>
	{/if}

	<section class="content-box p-5">
		<h2 class="mb-4 text-xl font-bold">Log journal entry</h2>

		{#if form?.journalSuccess}
			<p class="mb-4 border border-green-700 bg-green-100 p-3 text-sm text-green-900">
				Journal entry logged.
			</p>
		{/if}
		{#if form?.journalError}
			<p class="mb-4 border border-red-700 bg-red-100 p-3 text-sm text-red-900">
				{form.journalError}
			</p>
		{/if}

		<form method="post" action="?/createJournal" class="flex flex-col gap-4">
			<div class="flex flex-col gap-4 sm:flex-row sm:items-end">
				<div class="flex-1">
					<label for="duration" class="mb-1 block text-sm text-slate-600">Duration (minutes)</label>
					<input
						id="duration"
						name="durationInMinutes"
						type="number"
						min="1"
						required
						bind:value={durationInput}
						class="w-full border border-slate-700 bg-white/70 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
					/>
				</div>
				<button
					type="submit"
					class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
					disabled={!durationInput || parseInt(durationInput) <= 0 || !textInput.trim()}
				>
					Log entry
				</button>
			</div>
			<div>
				<label for="text" class="mb-1 block text-sm text-slate-600">What did you work on?</label>
				<textarea
					id="text"
					name="text"
					rows="2"
					required
					bind:value={textInput}
					class="w-full border border-slate-700 bg-white/70 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
				></textarea>
			</div>
		</form>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-2xl font-bold">Timeline</h2>

		{#if timeline.length === 0}
			<p class="border border-slate-500 bg-white/40 p-4 text-sm">
				No journal entries or reviews yet.
			</p>
		{:else}
			<div class="flex flex-col gap-4">
				{#each timeline as item, i (item.date.toISOString() + item.label)}
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
							<p class="text-sm text-slate-600">{item.detail}</p>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</main>
