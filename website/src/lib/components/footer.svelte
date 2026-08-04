<script lang="ts">
	import { resolve } from '$app/paths';
	import { fade } from 'svelte/transition';

	const repoURL = 'https://github.com/koeglike/hackxpansion';
	let { showGoUp = false }: { showGoUp?: boolean } = $props();
</script>

<footer transition:fade={{ duration: 100 }} class="w-full px-4 pb-2 text-sm text-slate-700">
	<hr class="w-full border-0 border-t border-slate-500" />
	<div
		class="grid min-h-16 w-full grid-cols-1 items-end gap-x-5 gap-y-1 sm:grid-cols-[1fr_auto_1fr]"
	>
		<p class="justify-self-center sm:justify-self-start">Hackxpansion, a Hack Club YSWS</p>
		{#if showGoUp}
			<button
				type="button"
				class="cursor-pointer justify-self-center hover:underline"
				onclick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
			>
				Go Up
			</button>
		{/if}
		<nav
			aria-label="Footer navigation"
			class="flex flex-wrap items-center justify-self-center gap-x-5 gap-y-1 sm:col-start-3 sm:justify-self-end"
		>
			<a href={resolve('/home/docs')} class="hover:underline">Docs</a>
			<a href={repoURL} target="_blank" rel="noopener noreferrer" class="hover:underline">
				GitHub
			</a>
			<a
				href="https://hackclub.com/"
				target="_blank"
				rel="noopener noreferrer"
				class="hover:underline"
			>
				Hack Club
			</a>
		</nav>
	</div>
</footer>
