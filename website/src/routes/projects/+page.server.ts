import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { createProject, ProjectMutationError } from '$lib/server/projects/mutations';
import { AriInboundError } from '$lib/server/ari/inbound';
import { canSubmit, ProjectSubmissionError, submitProjectToAri } from '$lib/server/projects/submit';
import { listSelectableCards, listCardDependencies } from '$lib/server/projects/queries';
import { eq } from 'drizzle-orm';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) {
		redirect(302, '/demo/hc');
	}

	const currentUser = locals.user;

	const projects = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			repoUrl: project.repoUrl,
			demoUrl: project.demoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			type: project.type,
			hackatimeProjects: project.hackatime_projects
		})
		.from(project)
		.where(eq(project.userId, locals.user.id));

	const projectsWithReadiness = await Promise.all(
		projects.map(async (project) => ({
			...project,
			readiness: await canSubmit({ projectId: project.id, userId: currentUser.id }),
			cardDependencies: project.type === 'app' ? await listCardDependencies(project.id) : []
		}))
	);

	const selectableCards = await listSelectableCards();

	return {
		projects: projectsWithReadiness,
		selectableCards
	};
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		if (!locals.user) {
			redirect(302, '/demo/hc');
		}

		const formData = await request.formData();

		try {
			const createdProject = await createProject({
				userId: locals.user.id,
				input: {
					title: stringFromForm(formData, 'title'),
					type: projectTypeFromForm(formData, 'type'),
					description: stringFromForm(formData, 'description'),
					repoUrl: stringFromForm(formData, 'repoUrl'),
					demoUrl: stringFromForm(formData, 'demoUrl'),
					thumbnailUrl: stringFromForm(formData, 'thumbnailUrl'),
					hackatimeProjects: stringListFromForm(formData, 'hackatimeProjects'),
					cardIds: stringListFromFormEntries(formData, 'cardIds')
				}
			});

			return { success: true, message: 'Project created.', projectId: createdProject.id };
		} catch (err) {
			return fail(getErrorStatus(err), {
				success: false,
				message: getErrorMessage(err),
				values: Object.fromEntries(formData.entries())
			});
		}
	},
	submit: async ({ locals, request }) => {
		if (!locals.user) {
			redirect(302, '/demo/hc');
		}

		const formData = await request.formData();
		const projectId = stringFromForm(formData, 'projectId');

		if (!projectId) {
			return fail(400, { success: false, message: 'Project ID is required.' });
		}

		try {
			const result = await submitProjectToAri({ projectId, userId: locals.user.id });

			return {
				success: true,
				message: `Submitted ${result.phase} review to Ari.`,
				projectId
			};
		} catch (err) {
			return fail(getErrorStatus(err), {
				success: false,
				message: getErrorMessage(err),
				projectId
			});
		}
	}
};

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}

function stringListFromForm(formData: FormData, key: string) {
	return stringFromForm(formData, key)
		.split(',')
		.map((value) => value.trim())
		.filter(Boolean);
}

function stringListFromFormEntries(formData: FormData, key: string) {
	return formData
		.getAll(key)
		.map((value) => (typeof value === 'string' ? value.trim() : ''))
		.filter(Boolean);
}

function projectTypeFromForm(formData: FormData, key: string): 'card' | 'app' {
	const value = stringFromForm(formData, key);
	return value === 'app' ? 'app' : 'card';
}

function getErrorStatus(err: unknown) {
	if (
		err instanceof ProjectMutationError ||
		err instanceof ProjectSubmissionError ||
		err instanceof AriInboundError
	) {
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
