<script lang="ts">
	import { resolve } from '$app/paths';
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	function value(name: keyof PageServerData['profile']) {
		return form?.profileValues?.[name] ?? data.profile[name] ?? '';
	}
</script>

<svelte:head>
	<title>Settings | Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Settings</h1>
		<p class="text-slate-600">Manage your Hackxpansion account and project information.</p>
	</header>

	<section aria-labelledby="project-settings" class="content-box p-5">
		<h2 id="project-settings" class="text-2xl font-bold">Project settings</h2>
		<p class="mt-2">
			Project details are managed individually from
			<a href={resolve('/home/projects')} class="underline">your projects</a>.
		</p>
	</section>

	<section aria-labelledby="submission-settings" class="content-box p-5">
		<h2 id="submission-settings" class="text-2xl font-bold">Submission details</h2>
		<p class="mt-2 text-slate-600">
			These details are used to pre-fill project review submissions.
		</p>

		{#if form?.message}
			<p
				class="mt-4 border p-3 text-sm"
				class:border-green-700={form.profileSuccess}
				class:bg-green-100={form.profileSuccess}
				class:text-green-900={form.profileSuccess}
				class:border-red-700={!form.profileSuccess}
				class:bg-red-100={!form.profileSuccess}
				class:text-red-900={!form.profileSuccess}
			>
				{form.message}
			</p>
		{/if}

		<form method="post" action="?/updateProfile" class="mt-5 grid gap-4 sm:grid-cols-2">
			<label class="flex flex-col gap-1 text-sm font-semibold sm:col-span-2">
				GitHub username
				<input
					name="githubUsername"
					value={value('githubUsername')}
					maxlength="39"
					pattern="[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?"
					autocomplete="username"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold sm:col-span-2">
				Address line 1
				<input
					name="addressLine1"
					value={value('addressLine1')}
					maxlength="200"
					autocomplete="address-line1"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold sm:col-span-2">
				Address line 2
				<input
					name="addressLine2"
					value={value('addressLine2')}
					maxlength="200"
					autocomplete="address-line2"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold">
				City
				<input
					name="addressCity"
					value={value('addressCity')}
					maxlength="100"
					autocomplete="address-level2"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold">
				State / province
				<input
					name="addressRegion"
					value={value('addressRegion')}
					maxlength="100"
					autocomplete="address-level1"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold">
				ZIP / postal code
				<input
					name="addressPostalCode"
					value={value('addressPostalCode')}
					maxlength="32"
					autocomplete="postal-code"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-semibold">
				Country
				<input
					name="addressCountry"
					value={value('addressCountry')}
					maxlength="100"
					autocomplete="country-name"
					class="border border-slate-500 bg-white p-2 font-normal"
				/>
			</label>
			<div class="sm:col-span-2">
				<button class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700">Save details</button>
			</div>
		</form>
	</section>

	<section aria-labelledby="account-settings" class="content-box p-5">
		<h2 id="account-settings" class="text-2xl font-bold">Account</h2>
		<p class="mt-2">Sign out of your Hack Club account on this device.</p>
		<form method="post" action={`${resolve('/home')}?/signOut`} class="mt-4">
			<button
				class="border border-slate-800 px-4 py-2 hover:bg-slate-800 hover:text-white cursor-pointer"
			>
				Sign out
			</button>
		</form>
	</section>
</main>
