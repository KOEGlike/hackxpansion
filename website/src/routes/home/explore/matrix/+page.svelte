<script lang="ts">
	import { resolve } from '$app/paths';
	import { getProjectProgress, getPublicProjectKey } from '$lib/projects/explore';
	import { E24_RESISTOR_VALUES, formatResistor } from '$lib/projects/resistors';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
	let hoveredMd0 = $state<number | null>(null);
	let hoveredMd1 = $state<number | null>(null);

	let projectsByPair = $derived.by(
		() =>
			new Map(
				data.projects
					.filter(
						(project): project is typeof project & { md0: number; md1: number } =>
							project.md0 !== null && project.md1 !== null
					)
					.map((project) => [`${project.md0}:${project.md1}`, project])
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

	function highlightClasses(md0: number, md1: number) {
		if (hoveredMd0 === md0 && hoveredMd1 === md1) {
			return 'ring-2 ring-slate-700 ring-inset brightness-110';
		}
		if (hoveredMd0 === md0 || hoveredMd1 === md1) {
			return 'ring-1 ring-slate-400 ring-inset brightness-105';
		}
		return '';
	}

	function clearHighlight() {
		hoveredMd0 = null;
		hoveredMd1 = null;
	}
</script>

<svelte:head>
	<title>Project Matrix | Hackxpansion</title>
</svelte:head>

<main class="flex min-w-0 flex-col gap-6 p-6 text-slate-800">
	<header class="mx-auto w-full max-w-5xl">
		<h1 class="text-4xl font-bold">Project matrix</h1>
		<p class="text-slate-600">
			Every MD0 and MD1 resistor pair. Select an occupied cell to open its public project page.
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

	<div class="content-box min-w-0 p-1">
		<div
			class="grid min-w-0 grid-cols-[1.5rem_minmax(0,1fr)_1.5rem] grid-rows-[1.5rem_minmax(0,1fr)_1.5rem]"
		>
			<p
				class="col-start-2 row-start-1 flex items-center justify-center text-base font-bold tracking-wide"
			>
				MD1
			</p>
			<div class="col-start-1 row-start-2 flex items-center justify-center" aria-hidden="true">
				<span class="writing-vertical-rl rotate-180 text-base font-bold tracking-wide">MD0</span>
			</div>
			<div class="col-start-2 row-start-2 max-h-[calc(100vh-18rem)] min-w-0 overflow-auto">
				<table class="border-separate border-spacing-1 text-xs" onmouseleave={clearHighlight}>
					<caption class="sr-only">Project occupancy by MD0 rows and MD1 columns</caption>
					<thead>
						<tr>
							<th class="invisible size-8 min-w-8 p-0" aria-hidden="true"></th>
							{#each E24_RESISTOR_VALUES as md1 (md1)}
								<th
									class="sticky top-0 z-20 size-8 min-w-8 max-w-8 border border-slate-400 p-0 align-middle font-medium {hoveredMd1 ===
									md1
										? 'bg-slate-300 text-slate-900'
										: 'bg-white'}"
								>
									<span class="whitespace-nowrap">{formatResistor(md1)}</span>
								</th>
							{/each}
						</tr>
					</thead>
					<tbody>
						{#each E24_RESISTOR_VALUES as md0 (md0)}
							<tr>
								<th
									class="sticky left-0 z-10 size-8 min-w-8 max-w-8 border border-slate-400 p-0 text-center font-medium {hoveredMd0 ===
									md0
										? 'bg-slate-300 text-slate-900'
										: 'bg-white'}"
								>
									{formatResistor(md0)}
								</th>
								{#each E24_RESISTOR_VALUES as md1 (md1)}
									{@const project = projectsByPair.get(`${md0}:${md1}`)}
									<td
										class="p-0"
										onmouseenter={() => {
											hoveredMd0 = md0;
											hoveredMd1 = md1;
										}}
									>
										{#if project}
											<a
												href={resolve(`/explore/${getPublicProjectKey(project)}`)}
												class="block size-8 border border-slate-400 transition-colors focus:outline-2 focus:outline-offset-1 focus:outline-slate-900 {cellClasses(
													project.status
												)} {highlightClasses(md0, md1)}"
												title={`${project.title} — MD0 ${formatResistor(md0)}, MD1 ${formatResistor(md1)}`}
												aria-label={`Open ${project.title}, MD0 ${formatResistor(md0)}, MD1 ${formatResistor(md1)}`}
												onfocus={() => {
													hoveredMd0 = md0;
													hoveredMd1 = md1;
												}}
												onblur={clearHighlight}
											></a>
										{:else}
											<span
												class="block size-8 border border-slate-400 bg-slate-100 transition {highlightClasses(
													md0,
													md1
												)}"
												title={`No project — MD0 ${formatResistor(md0)}, MD1 ${formatResistor(md1)}`}
												aria-label={`No project, MD0 ${formatResistor(md0)}, MD1 ${formatResistor(md1)}`}
											></span>
										{/if}
									</td>
								{/each}
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	</div>
</main>
