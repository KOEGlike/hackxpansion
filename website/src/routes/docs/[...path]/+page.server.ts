import { resolve } from '$app/paths';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = ({ params }) => {
	const suffix = params.path ? `/${params.path}` : '';
	redirect(308, `${resolve('/home/docs')}${suffix}`);
};
