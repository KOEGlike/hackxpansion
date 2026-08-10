<script lang="ts">
	import { inferGithubUsername, type UserSubmissionProfile } from '$lib/profile';
	import type { ProjectReviewPhase } from '$lib/projects/lifecycle';

	let {
		action,
		phase,
		profile,
		repoUrl,
		errorMessage,
		values
	}: {
		action: string;
		phase: ProjectReviewPhase;
		profile: UserSubmissionProfile;
		repoUrl: string | null;
		errorMessage?: string;
		values?: Record<string, string>;
	} = $props();

	function value(name: string, fallback: string | null = null) {
		return values?.[name] ?? fallback ?? '';
	}
</script>

<form method="post" {action} class="content-box flex w-full flex-col gap-6 p-5 sm:p-7">
	{#if errorMessage}
		<p class="border border-red-700 bg-red-100 p-3 text-sm text-red-900">{errorMessage}</p>
	{/if}

	<div>
		<h3 class="font-bold">Submission details</h3>
		<p class="text-sm text-slate-600">
			We save these details to pre-fill future review submissions. You can update them in Settings.
		</p>
	</div>

	<label class="flex flex-col gap-1 text-sm font-semibold">
		GitHub username
		<input
			name="githubUsername"
			value={value('githubUsername', profile.githubUsername ?? inferGithubUsername(repoUrl))}
			maxlength="39"
			pattern="[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?"
			autocomplete="username"
			class="border border-slate-500 bg-white p-2 font-normal"
		/>
	</label>

	<div class="grid gap-3 sm:grid-cols-2">
		<label class="flex flex-col gap-1 text-sm font-semibold sm:col-span-2">
			Address line 1
			<input
				name="addressLine1"
				value={value('addressLine1', profile.addressLine1)}
				maxlength="200"
				autocomplete="address-line1"
				required
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold sm:col-span-2">
			Address line 2 <span class="font-normal text-slate-500">(optional)</span>
			<input
				name="addressLine2"
				value={value('addressLine2', profile.addressLine2)}
				maxlength="200"
				autocomplete="address-line2"
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			City
			<input
				name="addressCity"
				value={value('addressCity', profile.addressCity)}
				maxlength="100"
				autocomplete="address-level2"
				required
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			State / province <span class="font-normal text-slate-500">(optional)</span>
			<input
				name="addressRegion"
				value={value('addressRegion', profile.addressRegion)}
				maxlength="100"
				autocomplete="address-level1"
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			ZIP / postal code
			<input
				name="addressPostalCode"
				value={value('addressPostalCode', profile.addressPostalCode)}
				maxlength="32"
				autocomplete="postal-code"
				required
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			Country
			<input
				name="addressCountry"
				value={value('addressCountry', profile.addressCountry)}
				maxlength="100"
				autocomplete="country-name"
				required
				class="border border-slate-500 bg-white p-2 font-normal"
			/>
		</label>
	</div>

	<fieldset class="flex flex-col gap-2">
		<legend class="font-bold"
			>How likely are you to recommend Hackxpansion? <span class="text-red-700">*</span></legend
		>
		<div class="grid grid-cols-6 gap-1 sm:grid-cols-11">
			{#each Array.from({ length: 11 }, (_, score) => score) as score (score)}
				<label
					class="flex cursor-pointer flex-col items-center gap-1 border border-slate-400 bg-white p-2 text-sm hover:bg-slate-100"
				>
					<input
						type="radio"
						name="nps"
						value={score}
						checked={value('nps') === String(score)}
						required
					/>
					{score}
				</label>
			{/each}
		</div>
		<div class="flex justify-between text-xs text-slate-500">
			<span>Not likely</span><span>Extremely likely</span>
		</div>
	</fieldset>

	<div class="flex flex-col gap-3">
		<h3 class="font-bold">Optional feedback</h3>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			How did you hear about Hackxpansion?
			<textarea
				name="howDidYouHear"
				maxlength="2000"
				class="min-h-20 border border-slate-500 bg-white p-2 font-normal"
				>{value('howDidYouHear')}</textarea
			>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			What are we doing well?
			<textarea
				name="whatAreWeDoingWell"
				maxlength="2000"
				class="min-h-20 border border-slate-500 bg-white p-2 font-normal"
				>{value('whatAreWeDoingWell')}</textarea
			>
		</label>
		<label class="flex flex-col gap-1 text-sm font-semibold">
			How can we improve?
			<textarea
				name="howCanWeImprove"
				maxlength="2000"
				class="min-h-20 border border-slate-500 bg-white p-2 font-normal"
				>{value('howCanWeImprove')}</textarea
			>
		</label>
	</div>

	<button type="submit" class="bg-slate-800 px-4 py-2 text-white hover:bg-slate-700">
		Submit {phase} review
	</button>
</form>
