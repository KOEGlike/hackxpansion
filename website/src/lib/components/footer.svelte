<script lang="ts">
	import { fade } from 'svelte/transition';

	const repoURL = 'https://github.com/koeglike/hackxpansion';
</script>

<footer
	transition:fade={{ duration: 100 }}
	class="flex h-20 w-full items-end justify-end p-5 text-sm text-slate-700"
>
	<a href={repoURL} target="_blank" rel="noopener noreferrer " class="hover:underline"> GitHub </a>
</footer>
