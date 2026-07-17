<script lang="ts">
	import { resolve } from '$app/paths';

	type ProjectFormData = {
		values?: Record<string, unknown>;
	} | null;

	type HackatimeProject = {
		name: string;
		totalSeconds: number;
	};

	type Tier = 'pro' | 'advanced' | 'basic';

	type ProjectFormValues = {
		title: string;
		type?: 'card' | 'app' | null;
		tier?: Tier | null;
		description?: string | null;
		repoUrl?: string | null;
		demoUrl?: string | null;
		thumbnailUrl?: string | null;
		hackatimeProjects?: string[] | null;
	};

	type ProjectTextField =
		| 'title'
		| 'type'
		| 'tier'
		| 'description'
		| 'repoUrl'
		| 'demoUrl'
		| 'thumbnailUrl';

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

	let selectedType = $state<'card' | 'app'>((formValue('type') as 'card' | 'app') || 'card');
	let isDropdownOpen = $state(false);
	let selectedTier = $state<string | null>((formValue('tier') as string) || null);
	let isTierDropdownOpen = $state(false);

	function selectType(value: 'card' | 'app') {
		selectedType = value;
		isDropdownOpen = false;
	}

	function selectTier(value: string | null) {
		selectedTier = value;
		isTierDropdownOpen = false;
	}

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
	<label class="flex flex-col gap-1">
		<span>Title *</span>
		<input name="title" required value={formValue('title')} {disabled} />
	</label>

	<div class="flex flex-col gap-0.5 relative">
		<span>Type *</span>
		<input type="hidden" name="type" value={selectedType} required />

		<button
			type="button"
			class="flex w-full items-center justify-between border border-slate-300 bg-white/70 px-3 py-2 text-left text-sm disabled:cursor-not-allowed disabled:bg-slate-100 disabled:opacity-50"
			onclick={() => (isDropdownOpen = !isDropdownOpen)}
			{disabled}
		>
			<span class="capitalize">{selectedType}</span>
		</button>

		{#if isDropdownOpen}
			<button
				type="button"
				tabindex="-1"
				aria-label="Close dropdown"
				class="fixed inset-0 z-10 h-full w-full cursor-default"
				onclick={() => (isDropdownOpen = false)}
			></button>

			<ul
				class="absolute top-[calc(100%+4px)] left-0 z-20 w-full border border-slate-300 bg-white shadow-lg"
			>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 capitalize transition-colors"
						onclick={() => selectType('card')}
					>
						Card
					</button>
				</li>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 capitalize transition-colors"
						onclick={() => selectType('app')}
					>
						App
					</button>
				</li>
			</ul>
		{/if}
	</div>

	<div class="flex flex-col gap-0.5 relative">
		<span>Tier</span>
		<input type="hidden" name="tier" value={selectedTier ?? ''} />

		<button
			type="button"
			class="flex w-full items-center justify-between border border-slate-300 bg-white/70 px-3 py-2 text-left text-sm disabled:cursor-not-allowed disabled:bg-slate-100 disabled:opacity-50"
			onclick={() => (isTierDropdownOpen = !isTierDropdownOpen)}
			{disabled}
		>
			<span class="capitalize">{selectedTier ?? 'Select a tier'}</span>
		</button>

		{#if isTierDropdownOpen}
			<button
				type="button"
				tabindex="-1"
				aria-label="Close dropdown"
				class="fixed inset-0 z-10 h-full w-full cursor-default"
				onclick={() => (isTierDropdownOpen = false)}
			></button>

			<ul
				class="absolute top-[calc(100%+4px)] left-0 z-20 w-full border border-slate-300 bg-white shadow-lg"
			>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 capitalize transition-colors"
						onclick={() => selectTier(null)}
					>
						None
					</button>
				</li>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 uppercase transition-colors"
						onclick={() => selectTier('pro')}
					>
						PRO
					</button>
				</li>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 capitalize transition-colors"
						onclick={() => selectTier('advanced')}
					>
						Advanced
					</button>
				</li>
				<li>
					<button
						type="button"
						class="w-full px-3 py-2 text-left text-sm hover:bg-slate-100 focus:bg-slate-100 active:bg-slate-200 capitalize transition-colors"
						onclick={() => selectTier('basic')}
					>
						Basic
					</button>
				</li>
			</ul>
		{/if}
	</div>

	<div class="md:col-span-2">
		{#if hackatimeError}
			<p class="text-sm text-amber-700">{hackatimeError}</p>
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
					class="border border-slate-300 bg-white/70 px-3 py-2 text-sm"
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
				class="grid w-full h-60 overflow-y-auto gap-1 border border-slate-300 p-2 md:grid-cols-2"
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
		<textarea name="description" rows="3" {disabled}>{formValue('description')}</textarea>
	</label>

	<label class="flex flex-col gap-1">
		<span>Repo URL</span>
		<input name="repoUrl" value={formValue('repoUrl')} {disabled} />
	</label>

	<label class="flex flex-col gap-1">
		<span>Thumbnail URL</span>
		<input name="thumbnailUrl" value={formValue('thumbnailUrl')} {disabled} />
	</label>

	<label class="flex flex-col gap-1 md:col-span-2">
		<span>Demo URL</span>
		<input
			name="demoUrl"
			placeholder="Required before build review"
			value={formValue('demoUrl')}
			{disabled}
		/>
	</label>

	<div class="flex items-center gap-4 md:col-span-2">
		<button
			class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-400"
			{disabled}
		>
			{submitLabel}
		</button>
		<a href={resolve(cancelPath)} class="text-sm text-slate-600 hover:underline">{cancelLabel}</a>
	</div>
</form>
