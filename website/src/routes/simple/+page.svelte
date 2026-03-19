<script lang="ts">
	import { asset, resolve } from '$app/paths';
	import DocsButton from '$lib/components/docs_button.svelte';
	import LandingSection from '$lib/components/landing_section.svelte';
	import { landingContent } from '$lib/content/content';
</script>

<DocsButton>
	<a class="anchor w-fit text-slate-700 hover:underline" href={resolve('/')}> simple </a>
	<p class="target text-slate-500">go to animated</p>
</DocsButton>

<div class="flex h-fit w-full flex-col items-center justify-start gap-15 px-10 pb-30 sm:gap-30">
	<div
		class="flex h-screen w-screen flex-col items-center justify-between gap-0 bg-cover bg-center"
		style:background-image="url({asset(`/renders/0000.webp`)})"
	>
		<LandingSection />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container">
		<div class="flex flex-col items-center gap-0">
			<h1 class="  text-6xl font-bold text-slate-700 sm:text-7xl">
				{landingContent.steps[0].title}
			</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">
				{landingContent.steps[0].description}
			</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/simple/module.webp')}`} alt="one module" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container sm:flex-row-reverse">
		<div class="flex flex-col items-center">
			<h1 class=" text-6xl font-bold text-slate-700 sm:text-7xl">
				{landingContent.steps[1].title}
			</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">
				{landingContent.steps[1].description}
			</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/ferris.webp')}`} alt="ferris" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container">
		<div class="flex flex-col items-center gap-0">
			<h1 class=" text-6xl font-bold text-slate-700 sm:text-7xl">
				{landingContent.steps[2].title}
			</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">
				{landingContent.steps[2].description}
			</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/simple/4modules.webp')}`} alt="four modules" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container sm:flex-row-reverse">
		<div class="flex flex-col items-center gap-0">
			<h1 class="text-6xl font-bold text-slate-700 sm:text-7xl">{landingContent.steps[3].title}</h1>
			<h2 class="text-xl font-normal text-slate-500 sm:text-2xl">
				{landingContent.steps[3].description}
			</h2>
		</div>
		<img
			class="w-64 object-cover"
			src={`${asset('/simple/exploaded-console.webp')}`}
			alt="console"
		/>
	</div>
</div>
