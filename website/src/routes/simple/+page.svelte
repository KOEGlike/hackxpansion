<script lang="ts">
	import { asset } from '$app/paths';
	import DocsButton from '$lib/components/docs_button.svelte';
	import Scroll from '$lib/components/scroll.svelte';
</script>

<DocsButton />

<div class="flex h-fit w-full flex-col items-center justify-start gap-30 px-10 pb-30">
	<div
		class="flex h-screen w-screen flex-col items-center justify-start gap-0 bg-cover bg-center"
		style:background-image="url({asset(`/renders/0000.webp`)})"
	>
		<div class="flex flex-col items-center gap-0 pt-75 sm:pt-90">
			<h1 class=" text-5xl font-bold text-slate-700 sm:text-7xl">Hackxpansion</h1>
			<h2 class="w-80 text-center text-xl font-normal text-slate-500 sm:w-100 sm:text-2xl">
				Make expansion cards, get a console to use them in!
			</h2>
			<Scroll extraclass="h-11 w-fit fill-slate-700 mt-100 sm:mt-85" />
		</div>
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container">
		<div class="flex flex-col items-center gap-0 pt-40 sm:pt-0">
			<h1 class="  text-6xl font-bold text-slate-700 sm:text-7xl">Step 1</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">Design your card!</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/simple/module.webp')}`} alt="one module" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container sm:flex-row-reverse">
		<div class="flex flex-col items-center">
			<h1 class=" text-6xl font-bold text-slate-700 sm:text-7xl">Step 2</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">Code a driver!</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/ferris.webp')}`} alt="ferris" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container">
		<div class="flex flex-col items-center gap-0 pt-40 sm:pt-0">
			<h1 class=" text-6xl font-bold text-slate-700 sm:text-7xl">Step 3</h1>
			<h2 class=" text-xl font-normal text-slate-500 sm:text-2xl">Make 3 more!</h2>
		</div>
		<img class="w-64 object-cover" src={`${asset('/simple/4modules.webp')}`} alt="four modules" />
	</div>
	<hr class="w-full border-dashed border-slate-500" />
	<div class="adaptive-container sm:flex-row-reverse">
		<div class="flex flex-col items-center gap-0 pt-15">
			<h1 class="text-6xl font-bold text-slate-700 sm:text-7xl">Step 4</h1>
			<h2 class="text-xl font-normal text-slate-500 sm:text-2xl">Submit and get the console!</h2>
		</div>
		<img
			class="w-64 object-cover"
			src={`${asset('/simple/exploaded-console.webp')}`}
			alt="console"
		/>
	</div>
</div>
