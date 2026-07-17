import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { AriInboundError } from '$lib/server/ari/inbound';
import {
	canSubmit,
	ProjectSubmissionError,
	submitProjectToAri,
	withdrawProjectFromAri
} from '$lib/server/projects/submit';
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
			tier: project.tier,
			hackatimeProjects: project.hackatime_projects,
			md1: project.md1,
			md2: project.md2
		})
		.from(project)
		.where(eq(project.userId, locals.user.id));

	const projectsWithReadiness = await Promise.all(
		projects.map(async (project) => ({
			...project,
			readiness: await canSubmit({ projectId: project.id, userId: currentUser.id })
		}))
	);

	return { projects: projectsWithReadiness };
};

export const actions: Actions = {
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
	},
	withdraw: async ({ locals, request }) => {
		if (!locals.user) {
			redirect(302, '/demo/hc');
		}

		const formData = await request.formData();
		const projectId = stringFromForm(formData, 'projectId');

		if (!projectId) {
			return fail(400, { success: false, message: 'Project ID is required.' });
		}

		try {
			await withdrawProjectFromAri({ projectId, userId: locals.user.id });

			return {
				success: true,
				message: 'Project withdrawn from Ari review.',
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

function getErrorStatus(err: unknown) {
	if (err instanceof ProjectSubmissionError || err instanceof AriInboundError) {
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
