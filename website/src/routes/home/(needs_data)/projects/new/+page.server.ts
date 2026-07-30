import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { createProject } from '$lib/server/projects/mutations';
import { loadProjectFormHackatimeProjects } from '$lib/server/projects/form';
import { handleProjectFormAction } from '$lib/server/projects/actions';
import { requireUser } from '$lib/server/guards';
import { resolve } from '$app/paths';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) return {} as never;
	const user = requireUser(locals);

	return loadProjectFormHackatimeProjects(
		user.slackId,
		'Could not load Hackatime projects. You can still create the project.'
	);
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		const user = requireUser(locals);
		const failure = await handleProjectFormAction({
			request,
			slackId: user.slackId,
			mutate: (input) => createProject({ userId: user.id, input })
		});
		if (failure) return failure;

		redirect(303, resolve('/home/projects'));
	}
};
