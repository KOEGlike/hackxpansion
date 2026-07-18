import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, journal, review } from '$lib/server/db/schema';
import { AriInboundError } from '$lib/server/ari/inbound';
import {
	canSubmit,
	ProjectSubmissionError,
	submitProjectToAri,
	withdrawProjectFromAri
} from '$lib/server/projects/submit';
import { getUserHackatimeProjectsWithStats } from '$lib/server/hackatime';
import { eq, sql } from 'drizzle-orm';

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

	const [journalStats, reviewStats, hackatimeStats] = await Promise.all([
		db
			.select({
				projectId: journal.projectId,
				totalJournalMinutes: sql<number>`COALESCE(SUM(${journal.durationInMinutes}), 0)`,
				journalCount: sql<number>`COUNT(${journal.id})`
			})
			.from(journal)
			.innerJoin(project, eq(journal.projectId, project.id))
			.where(eq(project.userId, currentUser.id))
			.groupBy(journal.projectId),
		db
			.select({
				projectId: review.projectId,
				reviewCount: sql<number>`COUNT(${review.id})`
			})
			.from(review)
			.innerJoin(project, eq(review.projectId, project.id))
			.where(eq(project.userId, currentUser.id))
			.groupBy(review.projectId),
		getUserHackatimeProjectsWithStats(currentUser.slackId).catch(() => [])
	]);

	const journalStatsByProject = new Map(journalStats.map((stats) => [stats.projectId, stats]));
	const reviewCountByProject = new Map(
		reviewStats.map((stats) => [stats.projectId, Number(stats.reviewCount)])
	);
	const hackatimeSecondsByProject = new Map(
		hackatimeStats.map((stats) => [stats.name, stats.totalSeconds])
	);

	const projectsWithReadiness = await Promise.all(
		projects.map(async (currentProject) => {
			const currentJournalStats = journalStatsByProject.get(currentProject.id);
			const totalJournalMinutes = Number(currentJournalStats?.totalJournalMinutes ?? 0);
			const hackatimeMinutes = Math.round(
				(currentProject.hackatimeProjects ?? []).reduce(
					(total, name) => total + (hackatimeSecondsByProject.get(name) ?? 0),
					0
				) / 60
			);

			return {
				...currentProject,
				totalJournalMinutes,
				journalCount: Number(currentJournalStats?.journalCount ?? 0),
				reviewCount: reviewCountByProject.get(currentProject.id) ?? 0,
				totalTrackedMinutes: totalJournalMinutes + hackatimeMinutes,
				readiness: await canSubmit({ projectId: currentProject.id, userId: currentUser.id })
			};
		})
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
