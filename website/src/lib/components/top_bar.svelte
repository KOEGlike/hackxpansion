<script lang="ts">
	import { resolve } from '$app/paths';

	type ModeSwitchRoute = '/' | '/simple';

	let {
		href,
		label,
		targetText
	}: {
		href: ModeSwitchRoute;
		label: string;
		targetText: string;
	} = $props();
</script>

<div class="fixed flex h-fit w-full flex-row-reverse items-start justify-between p-3">
	<!-- <a
		href="https://meko.fillout.com/hackxpansion"
		target="_blank"
		rel="noopener noreferrer"
		class="content-box p-2 text-xl hover:content-box-hover active:content-box">RSVP</a
	> -->
	<div class="flex h-fit w-fit flex-col items-end justify-end gap-2">
		<a
			href={resolve('/docs')}
			class="content-box p-2 text-xl hover:content-box-hover active:content-box">Docs</a
		>

		<a class="anchor w-fit text-slate-700 hover:underline" href={resolve(href)}>{label}</a>

		<p class="target text-slate-500">{targetText}</p>
	</div>
</div>
