<script lang="ts">
	import Scroll from '$lib/components/scroll.svelte';
	import { landingContent } from '$lib/content/content';
</script>

<div class="flex flex-col items-center justify-center gap-0 pt-[32vh]">
	<h1 class="max-w-full text-center text-[clamp(2.5rem,14vw,4.5rem)] font-bold text-slate-700">
		{landingContent.hero.title}
	</h1>
	<p
		class="w-[calc(100%-2rem)] max-w-100 text-center text-xl font-normal text-slate-600 sm:text-2xl"
	>
		{landingContent.hero.subtitle}
	</p>
</div>
<div class="mb-[11vh] flex flex-col items-center justify-center sm:mb-[6vh]">
	<Scroll extraclass="h-11 w-fit fill-slate-700" />
</div>
