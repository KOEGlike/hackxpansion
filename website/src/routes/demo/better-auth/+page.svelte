<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
</script>

<h1>Hi, use with nickname {data.user.given_name}!</h1>
<p>Your user ID is {data.user.id}.</p>
<p>
	Your are {#if !data.user.yswsEligible}
		not
	{/if} eligible for YSWSs.
</p>
<img src={data.user.image} alt="" />
<form method="post" action="?/signOut" use:enhance>
	<button class="rounded-md bg-blue-600 px-4 py-2 text-white transition hover:bg-blue-700"
		>Sign out</button
	>
</form>
