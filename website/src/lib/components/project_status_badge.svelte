<script lang="ts">
	type ProjectStatus =
		| 'not_submitted'
		| 'waiting_design'
		| 'rejected_design'
		| 'approved_design'
		| 'waiting_build'
		| 'rejected_build'
		| 'approved_build';

	let { status }: { status: ProjectStatus } = $props();

	const badges: Record<ProjectStatus, { label: string; classes: string }> = {
		not_submitted: {
			label: 'Not submitted',
			classes: 'bg-slate-200 text-slate-800'
		},
		waiting_design: {
			label: 'In design review',
			classes: 'bg-amber-200 text-amber-900'
		},
		rejected_design: {
			label: 'Design rejected',
			classes: 'bg-red-200 text-red-900'
		},
		approved_design: {
			label: 'Design approved',
			classes: 'bg-green-200 text-green-900'
		},
		waiting_build: {
			label: 'In build review',
			classes: 'bg-amber-200 text-amber-900'
		},
		rejected_build: {
			label: 'Build rejected',
			classes: 'bg-red-200 text-red-900'
		},
		approved_build: {
			label: 'Build approved',
			classes: 'bg-green-200 text-green-900'
		}
	};

	let badge = $derived(badges[status]);
</script>

<span class={`inline-block rounded-full px-3 py-0.5 text-xs font-semibold ${badge.classes}`}>
	{badge.label}
</span>
