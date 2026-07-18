<script lang="ts">
	import { asset } from '$app/paths';
	import { page } from '$app/state';
	import { landingContent } from '$lib/content/content';
	import './layout.css';

	const pageTitle = landingContent.hero.title;
	const pageDescription = landingContent.hero.subtitle;
	let canonicalUrl = $derived(page.url.href);
	let socialImage = $derived(new URL(asset('/renders/0000.webp'), page.url.origin).href);

	let { children } = $props();
</script>

<svelte:head>
	<title>{pageTitle}</title>
	<meta name="description" content={pageDescription} />
	<link rel="canonical" href={canonicalUrl} />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={canonicalUrl} />
	<meta property="og:title" content={pageTitle} />
	<meta property="og:description" content={pageDescription} />
	<meta property="og:image" content={socialImage} />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={pageTitle} />
	<meta name="twitter:description" content={pageDescription} />
	<meta name="twitter:image" content={socialImage} />

	<link rel="apple-touch-icon" sizes="180x180" href={asset('/favicon/apple-touch-icon.png')} />
	<link rel="icon" type="image/png" sizes="32x32" href={asset('/favicon/favicon-32x32.png')} />
	<link rel="icon" type="image/png" sizes="16x16" href={asset('/favicon/favicon-16x16.png')} />
</svelte:head>
<div class=" font-share-tech">
	{@render children()}
</div>
