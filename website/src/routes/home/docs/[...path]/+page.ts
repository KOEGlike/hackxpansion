import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

const documentModules = import.meta.glob('/src/lib/content/docs/**/*.md', {
	eager: true,
	query: '?raw',
	import: 'default'
}) as Record<string, string>;
const documentRoot = '/src/lib/content/docs/';

export const load = (({ params }) => {
	const requestedPath = params.path || 'index';
	const content =
		documentModules[`${documentRoot}${requestedPath}.md`] ??
		documentModules[`${documentRoot}${requestedPath}/index.md`];
	if (!content) error(404, 'Documentation page not found');

	const title = content.match(/^#\s+(.+)$/m)?.[1] ?? 'Documentation';
	return { content, title };
}) satisfies PageLoad;
