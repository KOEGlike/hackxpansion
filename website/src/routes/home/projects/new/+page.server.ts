import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { createProject } from '$lib/server/projects/mutations';
import {
	formValuesToObject,
	getErrorMessage,
	getInvalidProjectHackatimeProjects,
	getProjectMutationErrorStatus,
	loadProjectFormHackatimeProjects,
	projectInputFromForm
} from '$lib/server/projects/form';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) {
		redirect(302, '/demo/hc');
	}

	return loadProjectFormHackatimeProjects(
		locals.user.slackId,
		'Could not load Hackatime projects. You can still create the project.'
	);
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		if (!locals.user) {
			redirect(302, '/demo/hc');
		}

		const formData = await request.formData();
		const input = projectInputFromForm(formData);
		const invalidHackatimeProjects = await getInvalidProjectHackatimeProjects(
			locals.user.slackId,
			input.hackatimeProjects ?? []
		);

		if (invalidHackatimeProjects.length > 0) {
			return fail(422, {
				success: false,
				message: `Invalid Hackatime projects: ${invalidHackatimeProjects.join(', ')}`,
				values: formValuesToObject(formData)
			});
		}

		try {
			await createProject({
				userId: locals.user.id,
				input
			});
		} catch (err) {
			return fail(getProjectMutationErrorStatus(err), {
				success: false,
				message: getErrorMessage(err),
				values: formValuesToObject(formData)
			});
		}

		redirect(303, '/projects');
	}
};
