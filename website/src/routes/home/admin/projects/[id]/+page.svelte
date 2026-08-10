<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import Markdown from '$lib/components/markdown.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import { formatMinutes } from '$lib/projects/domain';
	import { formatResistor } from '$lib/projects/resistors';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });

	function reviewLabel(event: string) {
		return (
			{
				approved: 'Approved',
				changes: 'Changes requested',
				rejected: 'Rejected',
				reverted: 'Reverted',
				requeued: 'Requeued',
				fraud: 'Fraud detected'
			}[event] ?? event
		);
	}
</script>

<svelte:head>
	<title>{data.project.title} | Hackxpansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-6xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<a href={resolve('/home/admin/projects')} class="text-sm text-slate-600 hover:underline">
			&larr; Back to all projects
		</a>
		<div class="mt-2 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
			<div class="min-w-0">
				<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin project view</p>
				<div class="flex flex-wrap items-center gap-3">
					<h1 class="text-4xl font-bold">{data.project.title}</h1>
					<ProjectStatusBadge status={data.project.status} />
				</div>
				{#if data.project.description}
					<p class="mt-2 max-w-3xl text-slate-600">{data.project.description}</p>
				{/if}
			</div>
			{#if data.project.thumbnailUrl}
				<img
					src={data.project.thumbnailUrl}
					alt=""
					class="h-32 w-48 shrink-0 border border-slate-400 object-cover"
				/>
			{/if}
		</div>
	</header>

	<section class="grid gap-4 sm:grid-cols-3" aria-label="Project summary">
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Journal entries</p>
			<p class="mt-1 text-2xl font-bold">{data.stats.journalCount}</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Journaled time</p>
			<p class="mt-1 text-2xl font-bold">{formatMinutes(data.stats.totalJournalMinutes)}</p>
		</article>
		<article class="content-box p-4">
			<p class="text-xs uppercase tracking-wide text-slate-600">Currency paid</p>
			<p class="mt-1 flex items-center gap-1 text-2xl font-bold">
				<CoinIcon class="size-6" />
				{data.project.currencyPaidOut}
			</p>
		</article>
	</section>

	<div class="grid gap-6 lg:grid-cols-2">
		<section class="content-box p-5" aria-labelledby="project-details-heading">
			<h2 id="project-details-heading" class="text-2xl font-bold">Project details</h2>
			<dl class="mt-4 grid grid-cols-[auto_1fr] gap-x-5 gap-y-3 text-sm">
				<dt class="font-bold">ID</dt>
				<dd class="min-w-0 break-all">{data.project.id}</dd>
				<dt class="font-bold">Type</dt>
				<dd class="capitalize">{data.project.type}</dd>
				<dt class="font-bold">Tier</dt>
				<dd class="capitalize">{data.project.tier ?? 'Not assigned'}</dd>
				<dt class="font-bold">Design currency</dt>
				<dd>{data.project.designCurrencyAwarded ? 'Awarded' : 'Not awarded'}</dd>
				<dt class="font-bold">Approved design type</dt>
				<dd class="capitalize">{data.project.designApprovedType ?? 'Not approved'}</dd>
				<dt class="font-bold">Build currency</dt>
				<dd>{data.project.buildCurrencyAwarded ? 'Awarded' : 'Not awarded'}</dd>
				{#if data.project.type === 'card'}
					<dt class="font-bold">MD0 resistor</dt>
					<dd>{data.project.md0 == null ? 'Not assigned' : formatResistor(data.project.md0)}</dd>
					<dt class="font-bold">MD1 resistor</dt>
					<dd>{data.project.md1 == null ? 'Not assigned' : formatResistor(data.project.md1)}</dd>
				{/if}
				<dt class="font-bold">ARI submission</dt>
				<dd class="min-w-0 break-all">{data.project.activeAriExternalId ?? 'None active'}</dd>
				<dt class="font-bold">Hackatime projects</dt>
				<dd>{data.project.hackatimeProjects?.join(', ') || 'None linked'}</dd>
			</dl>
			<div class="mt-5 flex flex-wrap gap-3 text-sm">
				<!-- eslint-disable svelte/no-navigation-without-resolve -- validated external URLs -->
				{#if data.project.repoUrl}
					<a
						class="underline"
						href={data.project.repoUrl}
						target="_blank"
						rel="noopener noreferrer"
					>
						Repository
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
		</section>

		<section class="content-box p-5" aria-labelledby="owner-heading">
			<h2 id="owner-heading" class="text-2xl font-bold">Owner</h2>
			<div class="mt-4 flex items-center gap-4">
				{#if data.project.ownerImage}
					<img
						src={data.project.ownerImage}
						alt=""
						class="size-16 border border-slate-400 object-cover"
					/>
				{/if}
				<div class="min-w-0">
					<p class="truncate text-xl font-bold">{data.project.ownerName}</p>
					<p class="truncate text-sm text-slate-600">{data.project.ownerEmail}</p>
				</div>
			</div>
			<dl class="mt-5 grid grid-cols-[auto_1fr] gap-x-5 gap-y-3 text-sm">
				<dt class="font-bold">User ID</dt>
				<dd class="min-w-0 break-all">{data.project.userId}</dd>
				<dt class="font-bold">Slack ID</dt>
				<dd>{data.project.ownerSlackId}</dd>
				<dt class="font-bold">YSWS eligible</dt>
				<dd>{data.project.ownerYswsEligible ? 'Yes' : 'No'}</dd>
				<dt class="font-bold">Balance</dt>
				<dd class="flex items-center gap-1"><CoinIcon /> {data.project.ownerCurrency}</dd>
				<dt class="font-bold">Joined</dt>
				<dd>{dateFormatter.format(new Date(data.project.ownerCreatedAt))}</dd>
			</dl>
		</section>
	</div>

	<section class="flex flex-col gap-4" aria-labelledby="journals-heading">
		<div>
			<h2 id="journals-heading" class="text-2xl font-bold">Journals</h2>
			<p class="text-sm text-slate-600">All journal entries for this project, newest first.</p>
		</div>
		{#if data.journals.length === 0}
			<p class="content-box p-5">This project has no journal entries.</p>
		{:else}
			{#each data.journals as entry (entry.id)}
				<article class="content-box p-5">
					<div class="mb-3 flex flex-wrap items-baseline justify-between gap-2">
						<h3 class="font-bold">{formatMinutes(entry.durationInMinutes)}</h3>
						<p class="text-xs text-slate-500">
							{dateFormatter.format(new Date(entry.createdAt))}
							{#if new Date(entry.updatedAt).getTime() !== new Date(entry.createdAt).getTime()}
								· edited {dateFormatter.format(new Date(entry.updatedAt))}
							{/if}
						</p>
					</div>
					<Markdown text={entry.text} />
				</article>
			{/each}
		{/if}
	</section>

	<section class="flex flex-col gap-4" aria-labelledby="reviews-heading">
		<div>
			<h2 id="reviews-heading" class="text-2xl font-bold">Review history</h2>
			<p class="text-sm text-slate-600">Review events received from ARI.</p>
		</div>
		{#if data.reviews.length === 0}
			<p class="content-box p-5">This project has no review history.</p>
		{:else}
			{#each data.reviews as review (review.id)}
				<article class="content-box p-5">
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div>
							<h3 class="text-lg font-bold">{reviewLabel(review.event)}</h3>
							<p class="text-xs text-slate-500">ARI ID: {review.ariId}</p>
						</div>
						<p class="text-sm text-slate-600">
							{dateFormatter.format(new Date(review.receivedAt))}
						</p>
					</div>
					<div class="mt-4 grid gap-4 md:grid-cols-2">
						<div>
							<p class="text-xs font-bold uppercase text-slate-500">Note to maker</p>
							<p class="mt-1 whitespace-pre-wrap">{review.noteToMaker ?? 'No note provided.'}</p>
						</div>
						<div>
							<p class="text-xs font-bold uppercase text-slate-500">Audit details</p>
							<p class="mt-1 whitespace-pre-wrap">{review.auditNote ?? 'No audit note.'}</p>
							<p class="mt-2 text-sm text-slate-600">
								Approved time: {formatMinutes(review.approvedMinutes ?? 0)}
								{#if review.reviewer}
									· Reviewer: {review.reviewer.email}{/if}
							</p>
						</div>
					</div>
				</article>
			{/each}
		{/if}
	</section>
</main>
