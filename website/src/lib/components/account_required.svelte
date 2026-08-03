<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';

	let {
		message = 'You need an account to view this page and manage your Hackxpansion projects.'
	}: { message?: string } = $props();
	let returnTo = $derived(`${page.url.pathname}${page.url.search}`);
</script>

<main class="flex min-h-full items-center justify-center p-6 text-slate-800">
	<section
		class="content-box w-full max-w-xl p-8 text-center sm:p-12"
		aria-labelledby="account-required"
	>
		<p class="mb-2 text-sm font-bold uppercase tracking-widest text-slate-500">Account required</p>
		<h1 id="account-required" class="text-3xl font-bold sm:text-4xl">Sign In/Up to continue</h1>
		<p class="mx-auto mt-4 max-w-md text-lg text-slate-600">{message}</p>
		<form method="post" action={`${resolve('/')}?/signIn`} class="mt-7">
			<input type="hidden" name="returnTo" value={returnTo} />
			<button
				class="border border-slate-800 bg-slate-800 px-5 py-3 font-bold text-white hover:bg-transparent hover:text-slate-800 cursor-pointer"
			>
				Sign In/Up
			</button>
		</form>
	</section>
</main>
