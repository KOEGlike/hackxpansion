<script lang="ts">
	import Scroll from '$lib/components/scroll.svelte';
	import { landingContent } from '$lib/content/content';
</script>

<div class="flex flex-col items-center justify-center gap-0 pt-[32vh]">
	<h1 class="text-[3.4rem] font-bold text-slate-700 sm:text-7xl">{landingContent.hero.title}</h1>
	<h2 class="w-85 text-center text-xl font-normal text-slate-500 sm:w-100 sm:text-2xl">
		{landingContent.hero.subtitle}
	</h2>
</div>
<div class="mb-[11vh] flex flex-col items-center justify-center sm:mb-[6vh]">
	<Scroll extraclass="h-11 w-fit fill-slate-700" />
</div>
