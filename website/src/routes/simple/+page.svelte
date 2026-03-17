<script lang="ts">
	import { asset } from '$app/paths';
	import Scroll from '$lib/icons/scroll.svelte';
</script>

<div
	class="-z-10 flex h-screen w-screen flex-col items-center justify-start gap-0 bg-cover bg-center"
	style:background-image="url({asset(`/renders/0000.webp`)})"
>
	<div class="flex flex-col items-center gap-0 pt-75 sm:pt-90">
		<h1 class=" text-5xl font-bold text-slate-700 sm:text-7xl">Hackxpansion</h1>
		<h2 class="w-80 text-center text-xl font-normal text-slate-500 sm:w-100 sm:text-2xl">
			Make expansion cards, get a console to use them in!
		</h2>
		<Scroll extraclass="h-11 w-fit fill-slate-700 pt-10" />
	</div>
</div>
