import { error, redirect } from '@sveltejs/kit';
import { resolve } from '$app/paths';
import type { Actions, PageServerLoad } from './$types';
import { and, eq } from 'drizzle-orm';
import { isUuid } from '$lib/projects/domain';
import { getProjectSubmissionReadiness } from '$lib/projects/submission';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { requireUser } from '$lib/server/guards';
import { submitProjectAction } from '$lib/server/projects/actions';
import { getUserSubmissionProfile } from '$lib/server/user-profile';

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
			type: project.type,
			tier: project.tier,
			hackatimeProjects: project.hackatime_projects
		})
		.from(project)
		.where(and(eq(project.id, params.id), eq(project.userId, user.id)))
		.limit(1);

	if (!existingProject) error(404, 'Project not found');

	return {
		project: existingProject,
		readiness: getProjectSubmissionReadiness(existingProject, user.yswsEligible),
		profile: await getUserSubmissionProfile(user.id)
	};
};

export const actions: Actions = {
	submit: async ({ locals, params, request }) => {
		const user = requireUser(locals);
		if (!isUuid(params.id)) error(404, 'Project not found');

		const result = await submitProjectAction(params.id, user.id, await request.formData());
		if ('data' in result) return result;

		redirect(303, resolve(`/home/projects/${params.id}`));
	}
};
