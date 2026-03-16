<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';

	let { children } = $props();
</script>

<div class="flex h-screen w-screen flex-row gap-6 p-3">
	<div class="flex h-fit w-fit flex-col content-box p-2">
		<a
			class="text-2xl hover:underline"
			href={resolve('/docs')}
			class:underline={page.url.pathname === resolve('/docs')}
		>
			Documentation
		</a>

		<br class="h-2" />

		<a
			href={resolve('/docs/quickstart')}
			class="text-lg hover:underline"
			class:underline={page.url.pathname === resolve('/docs/quickstart')}
		>
			Getting Started
		</a>

		<a
			href={resolve('/docs/quickstart/first-card')}
			class="indent-4 hover:underline"
			class:underline={page.url.pathname === resolve('/docs/quickstart/first-card')}
		>
			First card
		</a>
		<a
			href={resolve('/docs/quickstart/first-driver')}
			class="indent-4 hover:underline"
			class:underline={page.url.pathname === resolve('/docs/quickstart/first-driver')}
		>
			First Driver
		</a>

		<a
			href={resolve('/docs/detailed')}
			class="text-lg hover:underline"
			class:underline={page.url.pathname === resolve('/docs/detailed')}
		>
			Detailed
		</a>
		<a
			href={resolve('/docs/detailed/card')}
			class="indent-4 hover:underline"
			class:underline={page.url.pathname === resolve('/docs/detailed/card')}
		>
			Card
		</a>
		<a
			href={resolve('/docs/detailed/api')}
			class="indent-4 hover:underline"
			class:underline={page.url.pathname === resolve('/docs/detailed/api')}
		>
			Driver API
		</a>
		<a
			href={resolve('/docs/detailed/device')}
			class="indent-4 hover:underline"
			class:underline={page.url.pathname === resolve('/docs/detailed/device')}
		>
			Device
		</a>
	</div>
	<div class="prose prose-lg prose-headings:text-slate-700 prose-p:text-slate-700">
		{@render children()}
	</div>
</div>
