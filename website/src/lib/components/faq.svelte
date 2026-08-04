<script lang="ts">
	import { landingContent } from '$lib/content/content';

	let openQuestions = $state(landingContent.faqs.map(() => false));
</script>

<section
	class="mx-auto w-full max-w-5xl px-4 py-20 text-slate-700 sm:px-10 sm:py-28"
	aria-labelledby="faq-heading"
>
	<h2 id="faq-heading" class="mb-8 text-4xl font-bold underline sm:text-5xl">FAQ</h2>

	<div class="border-y border-slate-500">
		{#each landingContent.faqs as faq, index (faq.question)}
			<div class:border-t={index > 0} class="border-slate-500">
				<button
					type="button"
					class="flex w-full cursor-pointer items-center justify-between gap-6 py-5 text-left text-xl font-semibold hover:underline"
					onclick={() => (openQuestions[index] = !openQuestions[index])}
					aria-expanded={openQuestions[index]}
					aria-controls={`faq-answer-${index}`}
				>
					<span>{faq.question}</span>
					<span aria-hidden="true" class="shrink-0">
						{openQuestions[index] ? 'v' : '>'}
					</span>
				</button>

				<div
					id={`faq-answer-${index}`}
					class="grid transition-[grid-template-rows,opacity] duration-300 ease-in-out"
					class:grid-rows-[1fr]={openQuestions[index]}
					class:grid-rows-[0fr]={!openQuestions[index]}
					class:opacity-100={openQuestions[index]}
					class:opacity-0={!openQuestions[index]}
				>
					<div class="min-h-0 overflow-hidden">
						<p class="max-w-3xl pb-5 text-base text-slate-600 sm:text-lg">{faq.answer}</p>
					</div>
				</div>
			</div>
		{/each}
	</div>
</section>
