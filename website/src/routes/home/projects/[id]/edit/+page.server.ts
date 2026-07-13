import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { editProject } from '$lib/server/projects/mutations';
import { canEditProject } from '$lib/server/projects/lifecycle';
import {
	formValuesToObject,
	getErrorMessage,
	getInvalidProjectHackatimeProjects,
	getProjectMutationErrorStatus,
	loadProjectFormHackatimeProjects,
	projectInputFromForm
} from '$lib/server/projects/form';
import { eq, and } from 'drizzle-orm';

export const load: PageServerLoad = async ({ locals, params }) => {
	if (!locals.user) {
		redirect(302, '/demo/hc');
	}

	const [existingProject] = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			repoUrl: project.repoUrl,
			demoUrl: project.demoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			hackatimeProjects: project.hackatime_projects
		})
		.from(project)
		.where(and(eq(project.id, params.id), eq(project.userId, locals.user.id)))
		.limit(1);

	if (!existingProject) {
		error(404, 'Project not found');
	}

	const hackatime = await loadProjectFormHackatimeProjects(
		locals.user.slackId,
		'Could not load Hackatime projects.'
	);

	return {
		project: existingProject,
		canEdit: canEditProject(existingProject.status),
		...hackatime
	};
};

export const actions: Actions = {
	edit: async ({ locals, params, request }) => {
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
			await editProject({
				projectId: params.id,
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
