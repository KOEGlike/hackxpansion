<script lang="ts">
	import { resolve } from '$app/paths';
	import CoinIcon from '$lib/components/coin_icon.svelte';
	import Markdown from '$lib/components/markdown.svelte';
	import ProjectStatusBadge from '$lib/components/project_status_badge.svelte';
	import { formatMinutes, isWaitingForReview } from '$lib/projects/domain';
	import { formatResistor } from '$lib/projects/resistors';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();
	let deleteDialog: HTMLDialogElement;
	const dateFormatter = new Intl.DateTimeFormat('en', { dateStyle: 'medium', timeStyle: 'short' });

	$effect(() => {
		if (form?.success === false && deleteDialog && !deleteDialog.open) deleteDialog.showModal();
	});

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

	function humanizeKey(key: string) {
		return key.replaceAll('_', ' ').replace(/^./, (character) => character.toUpperCase());
	}

	function objectEntries(value: object | null | undefined) {
		return Object.entries(value ?? {});
	}

	function formatValue(value: unknown) {
		if (value === null || value === undefined || value === '') return 'Not provided';
		if (typeof value === 'string') return value;
		if (typeof value === 'number' || typeof value === 'boolean') return String(value);
		return JSON.stringify(value, null, 2);
	}

	function formatJson(value: unknown) {
		return JSON.stringify(value, null, 2) ?? String(value);
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
							<div class="flex flex-wrap items-center gap-2">
								<h3 class="text-xl font-bold">{reviewLabel(review.event)}</h3>
								{#if review.rawPayload.decision}
									<span class="bg-slate-200 px-2 py-0.5 text-xs font-bold uppercase">
										{review.rawPayload.decision}
									</span>
								{/if}
							</div>
							<p class="text-xs text-slate-500">ARI ID: {review.ariId}</p>
						</div>
						<p class="text-sm text-slate-600">
							{dateFormatter.format(new Date(review.receivedAt))}
						</p>
					</div>

					<section class="mt-5" aria-label="Review identifiers">
						<dl class="grid gap-x-5 gap-y-2 text-sm sm:grid-cols-[auto_1fr_auto_1fr]">
							<dt class="font-bold">Review record</dt>
							<dd class="min-w-0 break-all">{review.id}</dd>
							<dt class="font-bold">Delivery ID</dt>
							<dd class="min-w-0 break-all">{review.deliveryId}</dd>
							<dt class="font-bold">External ID</dt>
							<dd class="min-w-0 break-all">{review.rawPayload.external_id}</dd>
							<dt class="font-bold">Event</dt>
							<dd>{review.rawPayload.event}</dd>
							<dt class="font-bold">Maker email</dt>
							<dd class="min-w-0 break-all">{review.rawPayload.maker.email}</dd>
							<dt class="font-bold">Maker Slack ID</dt>
							<dd>{review.rawPayload.maker.slack_id ?? 'Not provided'}</dd>
							<dt class="font-bold">Reviewer email</dt>
							<dd class="min-w-0 break-all">{review.reviewer?.email ?? 'Not provided'}</dd>
							<dt class="font-bold">Reviewer Slack ID</dt>
							<dd>{review.reviewer?.slack_id ?? 'Not provided'}</dd>
						</dl>
					</section>

					<section class="mt-5 grid gap-4 md:grid-cols-2" aria-label="Review notes">
						<div class="border border-slate-300 bg-white/40 p-4">
							<h4 class="text-xs font-bold uppercase text-slate-500">Note to maker</h4>
							<p class="mt-2 whitespace-pre-wrap">{review.noteToMaker ?? 'No note provided.'}</p>
						</div>
						<div class="border border-slate-300 bg-white/40 p-4">
							<h4 class="text-xs font-bold uppercase text-slate-500">Audit note</h4>
							<p class="mt-2 whitespace-pre-wrap">
								{review.auditNote ?? 'No audit note provided.'}
							</p>
						</div>
					</section>

					<section class="mt-5" aria-labelledby={`time-${review.id}`}>
						<h4 id={`time-${review.id}`} class="text-lg font-bold">Approved time</h4>
						<div class="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
							<div class="border border-slate-300 bg-white/40 p-3">
								<p class="text-xs uppercase text-slate-500">Total</p>
								<p class="text-lg font-bold">{formatMinutes(review.approvedMinutes ?? 0)}</p>
							</div>
							{#if review.minutesBreakdown}
								{#each objectEntries(review.minutesBreakdown) as [source, minutes] (source)}
									<div class="border border-slate-300 bg-white/40 p-3">
										<p class="text-xs uppercase text-slate-500">{humanizeKey(source)}</p>
										<p class="text-lg font-bold">{formatMinutes(Number(minutes))}</p>
									</div>
								{/each}
							{:else}
								<p class="self-center text-sm text-slate-600 sm:col-span-2 lg:col-span-4">
									No time breakdown was provided.
								</p>
							{/if}
						</div>
					</section>

					<section class="mt-5" aria-labelledby={`justification-${review.id}`}>
						<h4 id={`justification-${review.id}`} class="text-lg font-bold">Justification</h4>
						{#if review.justification && objectEntries(review.justification).length > 0}
							<dl class="mt-3 grid gap-3 md:grid-cols-2">
								{#each objectEntries(review.justification) as [key, value] (key)}
									<div class="border border-slate-300 bg-white/40 p-3">
										<dt class="text-xs font-bold uppercase text-slate-500">{humanizeKey(key)}</dt>
										<dd class="mt-1 whitespace-pre-wrap wrap-break-word">{formatValue(value)}</dd>
									</div>
								{/each}
							</dl>
						{:else}
							<p class="mt-2 text-sm text-slate-600">No justification was provided.</p>
						{/if}
					</section>

					<section class="mt-5" aria-labelledby={`fields-${review.id}`}>
						<h4 id={`fields-${review.id}`} class="text-lg font-bold">Reviewed fields</h4>
						{#if review.fields && review.fields.length > 0}
							<div class="mt-3 grid gap-3 md:grid-cols-2">
								{#each review.fields as field, index (`${field.key}:${index}`)}
									<div class="border border-slate-300 bg-white/40 p-3">
										<div class="flex flex-wrap items-baseline justify-between gap-2">
											<h5 class="font-bold">{field.label}</h5>
											<p class="text-xs text-slate-500">{field.key} · {field.type}</p>
										</div>
										<p class="mt-2 whitespace-pre-wrap wrap-break-word font-mono text-sm">
											{formatValue(field.value)}
										</p>
									</div>
								{/each}
							</div>
						{:else}
							<p class="mt-2 text-sm text-slate-600">No reviewed fields were provided.</p>
						{/if}
					</section>

					<section class="mt-5" aria-labelledby={`collaborators-${review.id}`}>
						<h4 id={`collaborators-${review.id}`} class="text-lg font-bold">Collaborators</h4>
						{#if review.collaborators && review.collaborators.length > 0}
							<div class="mt-3 grid gap-3 md:grid-cols-2">
								{#each review.collaborators as collaborator (collaborator.email)}
									<div class="border border-slate-300 bg-white/40 p-3">
										<h5 class="font-bold">{collaborator.name ?? collaborator.email}</h5>
										{#if collaborator.name}
											<p class="text-sm text-slate-600">{collaborator.email}</p>
										{/if}
										<dl class="mt-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
											<dt class="font-bold">Slack ID</dt>
											<dd>{collaborator.slack_id ?? 'Not provided'}</dd>
											<dt class="font-bold">Hackatime ID</dt>
											<dd>{collaborator.hackatime_id ?? 'Not provided'}</dd>
											<dt class="font-bold">Approved minutes</dt>
											<dd>{collaborator.approved_minutes ?? 'Not provided'}</dd>
											<dt class="font-bold">Approved hours</dt>
											<dd>{collaborator.approved_hours ?? 'Not provided'}</dd>
										</dl>
										{#if collaborator.minutes_breakdown}
											<div class="mt-3 border-t border-slate-300 pt-2 text-sm">
												{#each objectEntries(collaborator.minutes_breakdown) as [source, minutes] (source)}
													<p>{humanizeKey(source)}: {formatMinutes(Number(minutes))}</p>
												{/each}
											</div>
										{/if}
									</div>
								{/each}
							</div>
						{:else}
							<p class="mt-2 text-sm text-slate-600">No collaborators were included.</p>
						{/if}
					</section>

					<section class="mt-5" aria-labelledby={`fraud-${review.id}`}>
						<h4 id={`fraud-${review.id}`} class="text-lg font-bold">Fraud review</h4>
						{#if review.fraud}
							<p class="mt-2">
								Verdict:
								<span class="font-bold uppercase">{review.fraud.verdict}</span>
							</p>
							{#if review.fraud.checks.length > 0}
								<div class="mt-3 grid gap-3 md:grid-cols-2">
									{#each review.fraud.checks as check, index (`${check.email}:${index}`)}
										<div class="border border-slate-300 bg-white/40 p-3">
											<h5 class="font-bold">{check.email}</h5>
											<p class="text-sm text-slate-600">
												Slack ID: {check.slack_id ?? 'Not provided'} · Trust score: {check.trust_score}
											</p>
											<p class="mt-2 whitespace-pre-wrap">{check.justification}</p>
										</div>
									{/each}
								</div>
							{:else}
								<p class="mt-2 text-sm text-slate-600">No fraud checks were included.</p>
							{/if}
						{:else}
							<p class="mt-2 text-sm text-slate-600">No fraud review was included.</p>
						{/if}
					</section>

					<details class="mt-5 border border-slate-500 bg-slate-900 text-slate-100">
						<summary class="cursor-pointer px-4 py-3 font-bold hover:bg-slate-800">
							View raw JSON
						</summary>
						<pre class="max-h-128 overflow-auto border-t border-slate-600 p-4 text-xs"><code
								>{formatJson(review.rawPayload)}</code
							></pre>
					</details>
				</article>
			{/each}
		{/if}
	</section>

	<section class="border border-red-800 bg-red-50 p-5" aria-labelledby="danger-zone-heading">
		<h2 id="danger-zone-heading" class="text-2xl font-bold text-red-900">Danger zone</h2>
		<p class="mt-2 max-w-3xl text-sm text-red-950">
			Permanently delete this project and its journals and submission snapshots. Review audit
			records will remain without a project link, and the owner's currency balance will not change.
		</p>
		{#if isWaitingForReview(data.project.status)}
			<p class="mt-3 text-sm font-bold text-red-900">
				This project cannot be deleted while it is waiting for ARI review. Withdraw it first.
			</p>
		{/if}
		<button
			type="button"
			class="mt-4 border border-red-800 bg-red-800 px-4 py-2 font-bold text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:border-red-300 disabled:bg-red-300"
			disabled={isWaitingForReview(data.project.status)}
			onclick={() => deleteDialog.showModal()}
		>
			Delete project
		</button>
	</section>
</main>

<dialog
	bind:this={deleteDialog}
	class="m-auto w-[min(32rem,calc(100%-2rem))] border border-slate-900 bg-white p-0 text-slate-900 backdrop:bg-slate-950/60"
	aria-labelledby="delete-project-title"
>
	<div class="p-6">
		<h2 id="delete-project-title" class="text-2xl font-bold">Are you sure?</h2>
		<p class="mt-3">
			Deleting <strong>{data.project.title}</strong> is permanent. Its journals and submission snapshots
			will also be deleted.
		</p>
		{#if form?.message}
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
				<button
					class="border border-red-800 bg-red-800 px-4 py-2 font-bold text-white hover:bg-red-700"
				>
					Delete
				</button>
			</form>
		</div>
	</div>
</dialog>
