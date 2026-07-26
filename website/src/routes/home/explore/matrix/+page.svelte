<script lang="ts">
	import { resolve } from '$app/paths';
	import { getProjectProgress, getPublicProjectKey } from '$lib/projects/explore';
	import { E24_RESISTOR_VALUES, formatResistor } from '$lib/projects/resistors';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();

	let projectsByPair = $derived.by(
		() =>
			new Map(
				data.projects
					.filter(
						(project): project is typeof project & { md1: number; md2: number } =>
							project.md1 !== null && project.md2 !== null
					)
					.map((project) => [`${project.md1}:${project.md2}`, project])
			)
	);

	function cellClasses(status: PageServerData['projects'][number]['status']) {
		switch (getProjectProgress(status)) {
			case 'build_approved':
				return 'bg-slate-900 hover:bg-slate-800';
			case 'design_approved':
				return 'bg-slate-600 hover:bg-slate-500';
			case 'created':
				return 'bg-slate-300 hover:bg-slate-400';
		}
	}
</script>

<svelte:head>
	<title>Project Matrix | HackXPansion</title>
</svelte:head>

<main class="flex min-w-0 flex-col gap-6 p-6 text-slate-800">
	<header class="mx-auto w-full max-w-5xl">
		<h1 class="text-4xl font-bold">Project matrix</h1>
		<p class="text-slate-600">
			Every MD1 and MD2 resistor pair. Select an occupied cell to open its public project page.
		</p>
	</header>

	<section aria-label="Matrix legend" class="mx-auto flex w-full max-w-5xl flex-wrap gap-4 text-sm">
		<span class="flex items-center gap-2"
			><span class="size-4 border border-slate-300 bg-slate-100"></span>Empty</span
		>
		<span class="flex items-center gap-2"><span class="size-4 bg-slate-300"></span>Created</span>
		<span class="flex items-center gap-2"
			><span class="size-4 bg-slate-600"></span>Design approved</span
		>
		<span class="flex items-center gap-2"
			><span class="size-4 bg-slate-900"></span>Build approved</span
		>
	</section>

	<div class="content-box max-h-[calc(100vh-14rem)] min-w-0 overflow-auto p-3">
		<table class="border-separate border-spacing-1 text-xs">
			<caption class="sr-only">Project occupancy by MD1 rows and MD2 columns</caption>
			<thead>
				<tr>
					<th class="sticky top-0 left-0 z-30 bg-white px-2 py-1 text-right">MD1 \ MD2</th>
					{#each E24_RESISTOR_VALUES as md2 (md2)}
						<th class="sticky top-0 z-20 h-18 min-w-8 bg-white align-bottom font-medium">
							<span class="inline-block -rotate-45 whitespace-nowrap">{formatResistor(md2)}</span>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each E24_RESISTOR_VALUES as md1 (md1)}
					<tr>
						<th class="sticky left-0 z-10 bg-white px-2 py-1 text-right font-medium">
							{formatResistor(md1)}
						</th>
						{#each E24_RESISTOR_VALUES as md2 (md2)}
							{@const project = projectsByPair.get(`${md1}:${md2}`)}
							<td class="p-0">
								{#if project}
									<a
										href={resolve(`/explore/${getPublicProjectKey(project)}`)}
										class="block size-8 border border-slate-700 transition-colors focus:outline-2 focus:outline-offset-1 focus:outline-slate-900 {cellClasses(
											project.status
										)}"
										title={`${project.title} — MD1 ${formatResistor(md1)}, MD2 ${formatResistor(md2)}`}
										aria-label={`Open ${project.title}, MD1 ${formatResistor(md1)}, MD2 ${formatResistor(md2)}`}
									></a>
								{:else}
									<span
										class="block size-8 border border-slate-300 bg-slate-100"
										title={`No project — MD1 ${formatResistor(md1)}, MD2 ${formatResistor(md2)}`}
										aria-label={`No project, MD1 ${formatResistor(md1)}, MD2 ${formatResistor(md2)}`}
									></span>
								{/if}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</main>
