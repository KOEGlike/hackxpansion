<script lang="ts">
	import { resolve } from '$app/paths';
	import DropdownSelect from '$lib/components/dropdown_select.svelte';
	import type { ProjectTier, ProjectType } from '$lib/projects/domain';

	type ProjectFormData = {
		values?: Record<string, unknown>;
	} | null;

	type HackatimeProject = {
		name: string;
		totalSeconds: number;
	};

	type ProjectFormValues = {
		title: string;
		type?: ProjectType | null;
		tier?: ProjectTier;
		description?: string | null;
		repoUrl?: string | null;
		demoUrl?: string | null;
		thumbnailUrl?: string | null;
		hackatimeProjects?: string[] | null;
	};

	type ProjectTextField =
		'title' | 'type' | 'tier' | 'description' | 'repoUrl' | 'demoUrl' | 'thumbnailUrl';

	let {
		action,
		submitLabel,
		cancelPath,
		cancelLabel = 'Cancel',
		form,
		initialValues,
		hackatimeProjects,
		hackatimeError,
		disabled = false
	}: {
		action: string;
		submitLabel: string;
		cancelPath: '/home/projects';
		cancelLabel?: string;
		form: ProjectFormData | undefined;
		initialValues: ProjectFormValues;
		hackatimeProjects: HackatimeProject[];
		hackatimeError: string | null;
		disabled?: boolean;
	} = $props();

	let hackatimeProjectSearch = $state('');
	let normalizedHackatimeProjectSearch = $derived(hackatimeProjectSearch.trim().toLowerCase());
	let visibleHackatimeProjects = $derived(
		hackatimeProjects.filter((project) => matchesHackatimeProjectSearch(project.name))
	);
	let unavailableSelectedProjects = $derived(
		formValueList('hackatimeProjects').filter(
			(name) => !hackatimeProjects.some((project) => project.name === name)
		)
	);

	let selectedType = $state<string | null>((formValue('type') as string) || 'card');
	let selectedTier = $state<string | null>((formValue('tier') as string) || null);

	function formValue(key: ProjectTextField) {
		if (form?.values) {
			const value = form.values[key];
			if (typeof value === 'string') return value;
		}

		return initialTextValue(key);
	}

	function formValueList(key: string): string[] {
		if (form?.values) {
			const value = form.values[key];
			if (Array.isArray(value))
				return value.filter((entry): entry is string => typeof entry === 'string');
			if (typeof value === 'string' && value.trim()) return [value];
		}

		return initialValues.hackatimeProjects ?? [];
	}

	function isChecked(name: string) {
		return formValueList('hackatimeProjects').includes(name);
	}

	function matchesHackatimeProjectSearch(name: string) {
		return (
			!normalizedHackatimeProjectSearch ||
			name.toLowerCase().includes(normalizedHackatimeProjectSearch)
		);
	}

	function formatDuration(totalSeconds: number) {
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		if (hours > 0) return `${hours}h ${minutes}m`;
		return `${minutes}m`;
	}

	function initialTextValue(key: ProjectTextField) {
		switch (key) {
			case 'title':
				return initialValues.title;
			case 'type':
				return initialValues.type ?? 'card';
			case 'tier':
				return initialValues.tier ?? '';
			case 'description':
				return initialValues.description ?? '';
			case 'repoUrl':
				return initialValues.repoUrl ?? '';
			case 'demoUrl':
				return initialValues.demoUrl ?? '';
			case 'thumbnailUrl':
				return initialValues.thumbnailUrl ?? '';
		}
	}
</script>

<form method="post" {action} class="grid gap-4 md:grid-cols-2 w-full min-h-full">
	<label class="flex flex-col gap-0.5">
		<span>Title *</span>
		<input
			name="title"
			required
			value={formValue('title')}
			class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
			{disabled}
		/>
	</label>

	<DropdownSelect
		name="type"
		label="Type"
		options={[
			{ value: 'card', label: 'Card', class: 'capitalize' },
			{ value: 'app', label: 'App', class: 'capitalize' }
		]}
		bind:value={selectedType}
		required
		{disabled}
	/>

	<DropdownSelect
		name="tier"
		label="Tier"
		options={[
			{ value: null, label: 'None', class: 'capitalize' },
			{ value: 'pro', label: 'PRO', class: 'uppercase' },
			{ value: 'advanced', label: 'Advanced', class: 'capitalize' },
			{ value: 'basic', label: 'Basic', class: 'capitalize' }
		]}
		bind:value={selectedTier}
		placeholder="Select a tier"
		{disabled}
	/>

	<div class="md:col-span-2">
		<p class="mb-2 text-sm font-semibold">
			Hackatime projects <span class="font-normal text-slate-500">(optional)</span>
		</p>
		{#each unavailableSelectedProjects as projectName (projectName)}
			<input type="hidden" name="hackatimeProjects" value={projectName} {disabled} />
		{/each}
		{#if hackatimeError}
			<p class="text-sm text-amber-700">
				{hackatimeError} Existing selections will be preserved.
			</p>
		{/if}
		{#if hackatimeProjects.length === 0}
			<p class="text-sm text-slate-500">
				No Hackatime projects found. Make sure you have heartbeats logged in Hackatime.
			</p>
		{:else}
			<label class="mb-2 flex flex-col gap-1">
				<span class="text-sm font-semibold">Search Hackatime projects</span>
				<input
					type="search"
					bind:value={hackatimeProjectSearch}
					placeholder="Search by project name"
					autocomplete="off"
					class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
				/>
				<span class="text-xs text-slate-500">
					Showing {visibleHackatimeProjects.length} of {hackatimeProjects.length}
				</span>
			</label>

			{#if normalizedHackatimeProjectSearch && visibleHackatimeProjects.length === 0}
				<p class="mb-2 text-sm text-slate-500">
					No Hackatime projects match "{hackatimeProjectSearch}".
				</p>
			{/if}

			<div
				class="grid w-full h-60 overflow-y-auto gap-1 border border-slate-700 bg-white/70 p-2 md:grid-cols-2"
			>
				{#each hackatimeProjects as project (project.name)}
					<label
						hidden={!matchesHackatimeProjectSearch(project.name)}
						class="flex items-center gap-2 p-1 text-sm hover:bg-slate-100"
					>
						<input
							type="checkbox"
							name="hackatimeProjects"
							value={project.name}
							checked={isChecked(project.name)}
							{disabled}
						/>
						<span>{project.name}</span>
						<span class="text-slate-400">{formatDuration(project.totalSeconds)}</span>
					</label>
				{/each}
			</div>
		{/if}
	</div>

	<label class="flex flex-col gap-1 md:col-span-2">
		<span>Description</span>
		<textarea
			name="description"
			rows="3"
			class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
			{disabled}>{formValue('description')}</textarea
		>
	</label>

	<label class="flex flex-col gap-1">
		<span>Repo URL</span>
		<input
			type="url"
			name="repoUrl"
			value={formValue('repoUrl')}
			class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
			{disabled}
		/>
	</label>

	<label class="flex flex-col gap-1">
		<span>Thumbnail URL</span>
		<input
			type="url"
			name="thumbnailUrl"
			value={formValue('thumbnailUrl')}
			class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
			{disabled}
		/>
	</label>

	<label class="flex flex-col gap-1 md:col-span-2">
		<span>Demo URL</span>
		<input
			type="url"
			name="demoUrl"
			placeholder="Required before build review"
			value={formValue('demoUrl')}
			class="border border-slate-700 bg-white/70 px-3 py-2 text-sm"
			{disabled}
		/>
	</label>

	<div class="flex items-center gap-4 md:col-span-2">
		<button
			class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400 cursor-pointer"
			{disabled}
		>
			{submitLabel}
		</button>
		<a href={resolve(cancelPath)} class="text-sm text-slate-600 hover:underline">{cancelLabel}</a>
	</div>
</form>
