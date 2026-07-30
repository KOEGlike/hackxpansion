import { error, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { editProject } from '$lib/server/projects/mutations';
import { canEditProject } from '$lib/projects/lifecycle';
import { isUuid } from '$lib/projects/domain';
import { loadProjectFormHackatimeProjects } from '$lib/server/projects/form';
import { handleProjectFormAction } from '$lib/server/projects/actions';
import { requireUser } from '$lib/server/guards';
import { eq, and } from 'drizzle-orm';
import { resolve } from '$app/paths';

export const load: PageServerLoad = async ({ locals, params }) => {
	if (!locals.user) return {} as never;
	const user = requireUser(locals);
	if (!isUuid(params.id)) error(404, 'Project not found');

	const [existingProject] = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			repoUrl: project.repoUrl,
			demoUrl: project.demoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			hackatimeProjects: project.hackatime_projects,
			type: project.type,
			tier: project.tier
		})
		.from(project)
		.where(and(eq(project.id, params.id), eq(project.userId, user.id)))
		.limit(1);

	if (!existingProject) {
		error(404, 'Project not found');
	}

	const hackatime = await loadProjectFormHackatimeProjects(
		user.slackId,
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
		const user = requireUser(locals);
		if (!isUuid(params.id)) error(404, 'Project not found');
		const failure = await handleProjectFormAction({
			request,
			slackId: user.slackId,
			mutate: (input) => editProject({ projectId: params.id, userId: user.id, input })
		});
		if (failure) return failure;

		redirect(303, resolve('/home/projects'));
	}
};
