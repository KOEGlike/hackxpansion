<script lang="ts">
	import Markdown from '$lib/components/markdown.svelte';
	import { MAX_JOURNAL_DURATION_MINUTES } from '$lib/projects/journal';

	let {
		idPrefix,
		duration = $bindable(''),
		text = $bindable(''),
		tab = $bindable('write')
	}: {
		idPrefix: string;
		duration: string;
		text: string;
		tab: 'write' | 'preview';
	} = $props();
</script>

<div>
	<label for={`${idPrefix}-duration`} class="mb-1 block text-xs text-slate-500">
		Duration (minutes)
	</label>
	<input
		id={`${idPrefix}-duration`}
		name="durationInMinutes"
		type="number"
		min="1"
		max={MAX_JOURNAL_DURATION_MINUTES}
		step="1"
		required
		bind:value={duration}
		class="w-full border border-slate-700 bg-white/70 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
	/>
</div>

<div>
	<div class="mb-1 flex items-center gap-2">
		<label for={`${idPrefix}-text`} class="text-xs text-slate-500">What did you work on?</label>
		<div class="ml-auto flex gap-1" role="tablist" aria-label="Journal editor mode">
			<button
				type="button"
				role="tab"
				aria-selected={tab === 'write'}
				class="border px-2 py-0.5 text-xs {tab === 'write'
					? 'border-slate-700 bg-white/70'
					: 'border-transparent bg-transparent text-slate-500'}"
				onclick={() => (tab = 'write')}
			>
				Write
			</button>
			<button
				type="button"
				role="tab"
				aria-selected={tab === 'preview'}
				class="border px-2 py-0.5 text-xs {tab === 'preview'
					? 'border-slate-700 bg-white/70'
					: 'border-transparent bg-transparent text-slate-500'}"
				onclick={() => (tab = 'preview')}
			>
				Preview
			</button>
		</div>
	</div>

	<input type="hidden" name="text" value={text} />
	{#if tab === 'write'}
		<textarea
			id={`${idPrefix}-text`}
			rows="3"
			required
			bind:value={text}
			class="w-full border border-slate-700 bg-white/70 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-slate-500"
		></textarea>
	{:else}
		<div class="min-h-[4rem] border border-slate-700 bg-white/40 px-3 py-2 text-sm">
			{#if text.trim()}
				<Markdown {text} />
			{:else}
				<span class="text-slate-500">Nothing to preview.</span>
			{/if}
		</div>
	{/if}
</div>
