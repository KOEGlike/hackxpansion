<script lang="ts">
	import type { ActionData, PageServerData } from './$types';

	let { data, form }: { data: PageServerData; form: ActionData } = $props();

	let selectedType = $state<'card' | 'app'>(formTypeValue('type') ?? 'card');

	function formValue(key: string) {
		if (!form || !('values' in form) || !form.values) return '';

		const value = form.values[key];
		return typeof value === 'string' ? value : '';
	}

	function formTypeValue(key: string): 'card' | 'app' | undefined {
		const value = formValue(key);
		return value === 'app' ? 'app' : value === 'card' ? 'card' : undefined;
	}

	function formatResistor(ohms: number | null): string {
		if (ohms == null) return '—';
		if (ohms >= 1000) {
			const kilo = ohms / 1000;
			return `${kilo}k`;
		}
		return `${ohms}`;
	}
</script>

<svelte:head>
	<title>Projects · Hackxpansion</title>
</svelte:head>

<main class="mx-auto flex max-w-5xl flex-col gap-8 p-6 text-slate-800">
	<header>
		<h1 class="text-4xl font-bold">Projects</h1>
		<p class="text-slate-600">
			Create a card (hardware) or an app (software), then submit it to Ari.
		</p>
	</header>

	{#if form?.message}
		<p class="rounded-md border border-slate-700 bg-white/50 p-3 text-sm">
			{form.message}
		</p>
	{/if}

	<section class="rounded-lg border-2 border-slate-700 bg-slate-100/70 p-5">
		<h2 class="mb-4 text-2xl font-bold">Create project</h2>

		<form method="post" action="?/create" class="grid gap-4 md:grid-cols-2">
			<fieldset class="flex flex-col gap-1">
				<span>Type *</span>
				<label class="flex gap-2">
					<input
						type="radio"
						name="type"
						value="card"
						checked={selectedType === 'card'}
						onchange={() => (selectedType = 'card')}
					/>
					<span>Card (hardware)</span>
				</label>
				<label class="flex gap-2">
					<input
						type="radio"
						name="type"
						value="app"
						checked={selectedType === 'app'}
						onchange={() => (selectedType = 'app')}
					/>
					<span>App (software)</span>
				</label>
			</fieldset>

			<label class="flex flex-col gap-1">
				<span>Title *</span>
				<input name="title" required value={formValue('title')} class="rounded-md" />
			</label>

			<label class="flex flex-col gap-1">
				<span>Hackatime projects</span>
				<input
					name="hackatimeProjects"
					placeholder="project-one, project-two"
					value={formValue('hackatimeProjects')}
					class="rounded-md"
				/>
			</label>

			<label class="flex flex-col gap-1 md:col-span-2">
				<span>Description</span>
				<textarea name="description" rows="3" class="rounded-md"
					>{formValue('description')}</textarea
				>
			</label>

			<label class="flex flex-col gap-1">
				<span>Repo URL</span>
				<input name="repoUrl" value={formValue('repoUrl')} class="rounded-md" />
			</label>

			<label class="flex flex-col gap-1">
				<span>Thumbnail URL</span>
				<input name="thumbnailUrl" value={formValue('thumbnailUrl')} class="rounded-md" />
			</label>

			<label class="flex flex-col gap-1 md:col-span-2">
				<span>Demo URL {selectedType === 'app' ? '*' : ''}</span>
				<input
					name="demoUrl"
					placeholder={selectedType === 'app'
						? 'Required for apps (software)'
						: 'Required before build review'}
					required={selectedType === 'app'}
					value={formValue('demoUrl')}
					class="rounded-md"
				/>
			</label>

			{#if selectedType === 'app'}
				<label class="flex flex-col gap-1 md:col-span-2">
					<span>Required resources</span>
					<textarea
						name="requirements"
						rows="3"
						placeholder="e.g. Button&lt;A&gt;, I2C bus, 128x64 display"
						class="rounded-md">{formValue('requirements')}</textarea
					>
					<span class="text-xs text-slate-500">
						Describe the resource types this app depends on (interfaces, buses, peripherals).
					</span>
				</label>
			{/if}

			<div class="md:col-span-2">
				<button class="rounded-md bg-slate-800 px-4 py-2 text-white hover:bg-slate-700">
					Create {selectedType}
				</button>
			</div>
		</form>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-2xl font-bold">Your projects</h2>

		{#if data.projects.length === 0}
			<p class="rounded-lg border border-slate-500 bg-white/40 p-4">No projects yet.</p>
		{:else}
			{#each data.projects as project (project.id)}
				<article class="rounded-lg border-2 border-slate-700 bg-white/50 p-5">
					<div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
						<div class="flex gap-4">
							{#if project.thumbnailUrl}
								<img
									src={project.thumbnailUrl}
									alt=""
									class="h-20 w-20 rounded-md border border-slate-400 object-cover"
								/>
							{/if}

							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-xl font-bold">{project.title}</h3>
									<span
										class="rounded-full px-2 py-0.5 text-xs font-semibold {project.type === 'app'
											? 'bg-blue-100 text-blue-800'
											: 'bg-amber-100 text-amber-800'}"
									>
										{project.type === 'app' ? 'App · software' : 'Card · hardware'}
									</span>
								</div>
								<p class="text-sm text-slate-600">Status: {project.status}</p>
								{#if project.description}
									<p class="mt-2 max-w-2xl">{project.description}</p>
								{/if}
							</div>
						</div>

						<form method="post" action="?/submit">
							<input type="hidden" name="projectId" value={project.id} />
							<button
								class="rounded-md bg-blue-700 px-4 py-2 text-white hover:bg-blue-600 disabled:cursor-not-allowed disabled:bg-slate-400"
								disabled={!project.readiness.canSubmit}
							>
								Submit {project.readiness.phase ?? 'to'} review
							</button>
						</form>
					</div>

					<div class="mt-4 grid gap-2 text-sm md:grid-cols-2">
						{#if project.repoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.repoUrl} target="_blank" rel="noreferrer">Repo</a>
						{/if}
						{#if project.demoUrl}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- external URL -->
							<a class="underline" href={project.demoUrl} target="_blank" rel="noreferrer">Demo</a>
						{/if}
						<p>Hackatime: {project.hackatimeProjects?.join(', ') || 'none'}</p>
						{#if project.type === 'card'}
							<p>
								Module ID resistors: MD1 = {formatResistor(project.md1)}, MD2 =
								{formatResistor(project.md2)}
							</p>
						{/if}
					</div>

					{#if project.type === 'app'}
						<div class="mt-4 rounded-md bg-blue-50 p-3 text-sm text-blue-950">
							<p class="font-bold">Required resources:</p>
							{#if project.requirements}
								<p>{project.requirements}</p>
							{:else}
								<p>No required resources described.</p>
							{/if}
						</div>
					{/if}

					{#if !project.readiness.canSubmit}
						<div class="mt-4 rounded-md bg-amber-100 p-3 text-sm text-amber-950">
							<p class="font-bold">Before submitting:</p>
							<ul class="list-disc pl-5">
								{#each project.readiness.changes as change (change.field)}
									<li>{change.message}</li>
								{/each}
							</ul>
						</div>
					{/if}
				</article>
			{/each}
		{/if}
	</section>
</main>
