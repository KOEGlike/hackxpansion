import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { editProject, ProjectMutationError } from '$lib/server/projects/mutations';
import { canEditProject } from '$lib/server/projects/lifecycle';
import {
	getUserHackatimeProjectsWithStats,
	getUserHackatimeProjects,
	validateHackatimeProjectNames,
	type HackatimeProjectWithStats
} from '$lib/server/hackatime';
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

	let hackatimeProjects: HackatimeProjectWithStats[] = [];
	let hackatimeError: string | null = null;

	try {
		hackatimeProjects = await getUserHackatimeProjectsWithStats(locals.user.slackId);
	} catch {
		hackatimeError = 'Could not load Hackatime projects.';
	}

	return {
		project: existingProject,
		canEdit: canEditProject(existingProject.status),
		hackatimeProjects,
		hackatimeError
	};
};

export const actions: Actions = {
	edit: async ({ locals, params, request }) => {
		if (!locals.user) {
			redirect(302, '/demo/hc');
		}

		const formData = await request.formData();
		const hackatimeProjects = stringListFromForm(formData, 'hackatimeProjects');

		if (hackatimeProjects.length > 0) {
			try {
				const validProjects = await getUserHackatimeProjects(locals.user.slackId);
				const invalid = validateHackatimeProjectNames(hackatimeProjects, validProjects);
				if (invalid.length > 0) {
					return fail(422, {
						success: false,
						message: `Invalid Hackatime projects: ${invalid.join(', ')}`,
						values: formValuesToObject(formData)
					});
				}
			} catch {
				// API unreachable — skip validation
			}
		}

		try {
			await editProject({
				projectId: params.id,
				userId: locals.user.id,
				input: {
					title: stringFromForm(formData, 'title'),
					description: stringFromForm(formData, 'description'),
					repoUrl: stringFromForm(formData, 'repoUrl'),
					demoUrl: stringFromForm(formData, 'demoUrl'),
					thumbnailUrl: stringFromForm(formData, 'thumbnailUrl'),
					hackatimeProjects
				}
			});
		} catch (err) {
			return fail(getErrorStatus(err), {
				success: false,
				message: getErrorMessage(err),
				values: formValuesToObject(formData)
			});
		}

		redirect(303, '/projects');
	}
};

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}

function stringListFromForm(formData: FormData, key: string) {
	return formData
		.getAll(key)
		.map((value) => (typeof value === 'string' ? value : ''))
		.map((value) => value.trim())
		.filter(Boolean);
}

function formValuesToObject(formData: FormData): Record<string, string | string[]> {
	const values: Record<string, string | string[]> = {};
	for (const [key, value] of formData.entries()) {
		const strValue = String(value);
		if (key in values) {
			const existing = values[key];
			if (Array.isArray(existing)) {
				existing.push(strValue);
			} else {
				values[key] = [existing, strValue];
			}
		} else {
			values[key] = strValue;
		}
	}
	return values;
}

function getErrorStatus(err: unknown) {
	if (err instanceof ProjectMutationError) {
		return err.status;
	}

	return 500;
}

function getErrorMessage(err: unknown) {
	if (err instanceof Error) {
		return err.message;
	}

	return 'Something went wrong.';
}
