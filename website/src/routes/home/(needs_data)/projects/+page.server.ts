import { fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, journal, review } from '$lib/server/db/schema';
import { getUserHackatimeProjectsWithStats } from '$lib/server/hackatime';
import { submitProjectAction, withdrawProjectAction } from '$lib/server/projects/actions';
import { requireUser } from '$lib/server/guards';
import { getProjectSubmissionReadiness } from '$lib/projects/submission';
import { isUuid } from '$lib/projects/domain';
import { sumHackatimeMinutes } from '$lib/projects/time';
import { eq, sql } from 'drizzle-orm';

export const load: PageServerLoad = async ({ locals }) => {
	const currentUser = requireUser(locals);

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
			currencyPaidOut: project.currencyPaidOut,
			hackatimeProjects: project.hackatime_projects,
			md1: project.md1,
			md2: project.md2
		})
		.from(project)
		.where(eq(project.userId, currentUser.id));

	const [journalStats, reviewStats, hackatimeResult] = await Promise.all([
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
		getUserHackatimeProjectsWithStats(currentUser.slackId).then(
			(stats) => ({ stats, error: null }),
			() => ({ stats: [], error: 'Hackatime totals are temporarily unavailable.' })
		)
	]);

	const journalStatsByProject = new Map(journalStats.map((stats) => [stats.projectId, stats]));
	const reviewCountByProject = new Map(
		reviewStats.map((stats) => [stats.projectId, Number(stats.reviewCount)])
	);
	const projectsWithReadiness = projects.map((currentProject) => {
		const currentJournalStats = journalStatsByProject.get(currentProject.id);
		const totalJournalMinutes = Number(currentJournalStats?.totalJournalMinutes ?? 0);
		const hackatimeMinutes = sumHackatimeMinutes(
			currentProject.hackatimeProjects,
			hackatimeResult.stats
		);

		return {
			...currentProject,
			totalJournalMinutes,
			journalCount: Number(currentJournalStats?.journalCount ?? 0),
			reviewCount: reviewCountByProject.get(currentProject.id) ?? 0,
			totalTrackedMinutes: totalJournalMinutes + hackatimeMinutes,
			readiness: getProjectSubmissionReadiness(currentProject)
		};
	});

	return { projects: projectsWithReadiness, hackatimeError: hackatimeResult.error };
};

export const actions: Actions = {
	submit: async ({ locals, request }) => {
		const user = requireUser(locals);
		const formData = await request.formData();
		const projectId = stringFromForm(formData, 'projectId');

		if (!isUuid(projectId)) {
			return fail(400, { success: false, message: 'A valid project ID is required.' });
		}

		return submitProjectAction(projectId, user.id);
	},
	withdraw: async ({ locals, request }) => {
		const user = requireUser(locals);
		const formData = await request.formData();
		const projectId = stringFromForm(formData, 'projectId');

		if (!isUuid(projectId)) {
			return fail(400, { success: false, message: 'A valid project ID is required.' });
		}

		return withdrawProjectAction(projectId, user.id);
	}
};

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}
