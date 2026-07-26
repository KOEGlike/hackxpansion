import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, journal, review } from '$lib/server/db/schema';
import { and, desc, eq, sql } from 'drizzle-orm';
import { canEditProject } from '$lib/projects/lifecycle';
import { getProjectSubmissionReadiness } from '$lib/projects/submission';
import { isUuid } from '$lib/projects/domain';
import { sumHackatimeMinutes } from '$lib/projects/time';
import { getUserHackatimeProjectsWithStats } from '$lib/server/hackatime';
import {
	handleJournalAction,
	submitProjectAction,
	withdrawProjectAction
} from '$lib/server/projects/actions';
import { createProjectJournal, editProjectJournal } from '$lib/server/projects/journals';
import { requireUser } from '$lib/server/guards';

export const load: PageServerLoad = async ({ locals, params }) => {
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
			currencyPaidOut: project.currencyPaidOut,
			md0: project.md0,
			md1: project.md1,
			hackatimeProjects: project.hackatime_projects
		})
		.from(project)
		.where(and(eq(project.id, params.id), eq(project.userId, user.id)))
		.limit(1);

	if (!existingProject) error(404, 'Project not found');

	const [journals, reviews, stats, hackatimeResult] = await Promise.all([
		db
			.select()
			.from(journal)
			.where(eq(journal.projectId, params.id))
			.orderBy(desc(journal.createdAt)),
		db
			.select({
				id: review.id,
				event: review.event,
				receivedAt: review.receivedAt,
				noteToMaker: review.noteToMaker
			})
			.from(review)
			.where(eq(review.projectId, params.id))
			.orderBy(desc(review.receivedAt)),
		db
			.select({
				totalJournalMinutes: sql<number>`COALESCE(SUM(${journal.durationInMinutes}), 0)`,
				journalCount: sql<number>`COUNT(${journal.id})`
			})
			.from(project)
			.leftJoin(journal, eq(journal.projectId, project.id))
			.where(eq(project.id, params.id))
			.groupBy(project.id)
			.then((rows) => rows[0]),
		getUserHackatimeProjectsWithStats(user.slackId).then(
			(entries) => ({ entries, error: null }),
			() => ({ entries: [], error: 'Hackatime totals are temporarily unavailable.' })
		)
	]);

	return {
		project: existingProject,
		journals,
		reviews,
		stats: {
			totalJournalMinutes: Number(stats?.totalJournalMinutes ?? 0),
			journalCount: Number(stats?.journalCount ?? 0)
		},
		readiness: getProjectSubmissionReadiness(existingProject),
		canEdit: canEditProject(existingProject.status),
		hackatime: {
			minutes: sumHackatimeMinutes(existingProject.hackatimeProjects, hackatimeResult.entries),
			error: hackatimeResult.error
		}
	};
};

export const actions: Actions = {
	createJournal: async ({ locals, params, request }) => {
		const user = requireUser(locals);
		if (!isUuid(params.id)) return fail(404, { journalError: 'Project not found.' });
		return handleJournalAction(request, (input) =>
			createProjectJournal({ projectId: params.id, userId: user.id, input })
		);
	},

	editJournal: async ({ locals, params, request }) => {
		const user = requireUser(locals);
		if (!isUuid(params.id)) return fail(404, { journalError: 'Project not found.' });
		return handleJournalAction(request, (input, formData) => {
			const journalId = formData.get('journalId');
			return editProjectJournal({
				projectId: params.id,
				journalId: typeof journalId === 'string' ? journalId : '',
				userId: user.id,
				input
			});
		});
	},

	submit: async ({ locals, params }) => {
		const user = requireUser(locals);
		if (!isUuid(params.id)) return fail(404, { success: false, message: 'Project not found.' });
		return submitProjectAction(params.id, user.id);
	},

	withdraw: async ({ locals, params }) => {
		const user = requireUser(locals);
		if (!isUuid(params.id)) return fail(404, { success: false, message: 'Project not found.' });
		return withdrawProjectAction(params.id, user.id);
	}
};
