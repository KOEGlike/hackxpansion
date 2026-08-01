<script lang="ts">
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	const admins = $derived(data.users.filter((user) => user.isAdmin));
	const members = $derived(data.users.filter((user) => !user.isAdmin));
</script>

<svelte:head>
	<title>Users | HackXPansion Admin</title>
</svelte:head>

<main class="mx-auto flex max-w-6xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<p class="text-sm font-bold uppercase tracking-widest text-slate-500">Admin</p>
		<h1 class="text-4xl font-bold">Users</h1>
		<p class="text-slate-600">Grant admin access to existing HackXPansion users.</p>
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

	<section class="flex flex-col gap-4" aria-labelledby="members-heading">
		<div>
			<h2 id="members-heading" class="text-2xl font-bold">Users</h2>
			<p class="text-sm text-slate-600">These accounts do not currently have admin access.</p>
		</div>
		{#if members.length === 0}
			<p class="content-box p-5">Every user is already an admin.</p>
		{:else}
			<div class="grid gap-4 lg:grid-cols-2">
				{#each members as user (user.id)}
					<article
						class="content-box flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between"
					>
						<div class="min-w-0">
							<h3 class="truncate text-xl font-bold">{user.name}</h3>
							<p class="truncate text-sm text-slate-600">{user.email}</p>
							<p class="text-xs text-slate-500">Slack ID: {user.slackId}</p>
						</div>
						<form method="post" action="?/promote">
							<input type="hidden" name="userId" value={user.id} />
							<button class="bg-slate-800 px-4 py-2 font-bold text-white hover:bg-slate-700">
								Make admin
							</button>
						</form>
					</article>
				{/each}
			</div>
		{/if}
	</section>

	<section class="flex flex-col gap-4" aria-labelledby="admins-heading">
		<h2 id="admins-heading" class="text-2xl font-bold">Current admins</h2>
		<div class="grid gap-4 lg:grid-cols-2">
			{#each admins as user (user.id)}
				<article
					class="content-box flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between"
				>
					<div class="min-w-0">
						<h3 class="truncate text-xl font-bold">{user.name}</h3>
						<p class="truncate text-sm text-slate-600">{user.email}</p>
						<p class="text-xs text-slate-500">Slack ID: {user.slackId}</p>
					</div>
					{#if user.isProtectedAdmin}
						<span class="self-start bg-green-200 px-2 py-1 text-xs font-bold uppercase">
							Protected admin
						</span>
					{:else}
						<form method="post" action="?/demote">
							<input type="hidden" name="userId" value={user.id} />
							<button
								class="border border-red-800 px-4 py-2 font-bold text-red-900 hover:bg-red-100"
							>
								Remove admin
							</button>
						</form>
					{/if}
				</article>
			{/each}
		</div>
	</section>
</main>
