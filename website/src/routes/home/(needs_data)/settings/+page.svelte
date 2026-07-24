<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<svelte:head>
	<title>Settings | HackXPansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Settings</h1>
		<p class="text-slate-600">Manage your HackXPansion account and project information.</p>
	</header>

	<section aria-labelledby="project-settings" class="content-box p-5">
		<h2 id="project-settings" class="text-2xl font-bold">Project settings</h2>
		<p class="mt-2">
			Project details are managed individually from
			<a href={resolve('/home/projects')} class="underline">your projects</a>.
		</p>
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
