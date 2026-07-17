import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, journal, review } from '$lib/server/db/schema';
import { eq, and, desc, sql } from 'drizzle-orm';
import { canEditProject } from '$lib/server/projects/lifecycle';
import {
	canSubmit,
	submitProjectToAri,
	withdrawProjectFromAri,
	ProjectSubmissionError
} from '$lib/server/projects/submit';
import { AriInboundError } from '$lib/server/ari/inbound';
import { getUserHackatimeProjectsWithStats } from '$lib/server/hackatime';

export const load: PageServerLoad = async ({ locals, params }) => {
	if (!locals.user) {
		error(401, 'Unauthorized');
	}

	const [existingProject] = await db
		.select()
		.from(project)
		.where(and(eq(project.id, params.id), eq(project.userId, locals.user.id)))
		.limit(1);

	if (!existingProject) {
		error(404, 'Project not found');
	}

	const journals = await db
		.select()
		.from(journal)
		.where(eq(journal.projectId, params.id))
		.orderBy(desc(journal.createdAt));

	const reviews = await db
		.select()
		.from(review)
		.where(eq(review.projectId, params.id))
		.orderBy(desc(review.receivedAt));

	const [stats] = await db
		.select({
			totalJournalMinutes: sql<number>`COALESCE(SUM(${journal.durationInMinutes}), 0)`,
			journalCount: sql<number>`COUNT(${journal.id})`,
			totalApprovedMinutes: sql<number>`COALESCE(SUM(${review.approvedMinutes}) FILTER (WHERE ${review.event} = 'approved'), 0)`,
			reviewCount: sql<number>`COUNT(${review.id})`
		})
		.from(project)
		.leftJoin(journal, eq(journal.projectId, project.id))
		.leftJoin(review, eq(review.projectId, project.id))
		.where(eq(project.id, params.id))
		.groupBy(project.id);

	const [readiness, hackatimeMinutes] = await Promise.all([
		canSubmit({ projectId: params.id, userId: locals.user.id }),
		getHackatimeMinutesForProject(existingProject.hackatime_projects, locals.user.slackId)
	]);

	return {
		project: existingProject,
		journals,
		reviews,
		stats: {
			totalJournalMinutes: Number(stats?.totalJournalMinutes ?? 0),
			journalCount: Number(stats?.journalCount ?? 0),
			totalApprovedMinutes: Number(stats?.totalApprovedMinutes ?? 0),
			reviewCount: Number(stats?.reviewCount ?? 0)
		},
		readiness,
		canEdit: canEditProject(existingProject.status),
		hackatimeMinutes
	};
};

async function getHackatimeMinutesForProject(
	hackatimeProjects: string[] | null,
	slackId: string
): Promise<number> {
	if (!hackatimeProjects || hackatimeProjects.length === 0) return 0;

	try {
		const allStats = await getUserHackatimeProjectsWithStats(slackId);
		const totalSeconds = allStats
			.filter((p) => hackatimeProjects.includes(p.name))
			.reduce((sum, p) => sum + p.totalSeconds, 0);
		return Math.round(totalSeconds / 60);
	} catch {
		return 0;
	}
}

function assertCanEditJournal(projectStatus: string) {
	if (!canEditProject(projectStatus as Parameters<typeof canEditProject>[0])) {
		error(403, 'Cannot modify journals while the project is under review.');
	}
}

export const actions: Actions = {
	createJournal: async ({ locals, params, request }) => {
		if (!locals.user) {
			error(401, 'Unauthorized');
		}

		const formData = await request.formData();
		const durationStr = formData.get('durationInMinutes');
		const textVal = formData.get('text');

		if (!durationStr || typeof durationStr !== 'string') {
			return fail(400, { journalError: 'Duration is required.' });
		}

		if (!textVal || typeof textVal !== 'string' || !textVal.trim()) {
			return fail(400, { journalError: 'Journal text is required.' });
		}

		const duration = parseInt(durationStr, 10);
		if (isNaN(duration) || duration <= 0) {
			return fail(400, { journalError: 'Duration must be a positive number.' });
		}

		const [existingProject] = await db
			.select({ id: project.id, status: project.status })
			.from(project)
			.where(and(eq(project.id, params.id), eq(project.userId, locals.user.id)))
			.limit(1);

		if (!existingProject) {
			error(404, 'Project not found');
		}

		assertCanEditJournal(existingProject.status);

		await db.insert(journal).values({
			durationInMinutes: duration,
			text: textVal.trim(),
			projectId: params.id
		});

		return { journalSuccess: true };
	},

	editJournal: async ({ locals, params, request }) => {
		if (!locals.user) {
			error(401, 'Unauthorized');
		}

		const formData = await request.formData();
		const journalId = formData.get('journalId');
		const durationStr = formData.get('durationInMinutes');
		const textVal = formData.get('text');

		if (!journalId || typeof journalId !== 'string') {
			return fail(400, { journalError: 'Journal ID is required.' });
		}

		if (!durationStr || typeof durationStr !== 'string') {
			return fail(400, { journalError: 'Duration is required.' });
		}

		if (!textVal || typeof textVal !== 'string' || !textVal.trim()) {
			return fail(400, { journalError: 'Journal text is required.' });
		}

		const duration = parseInt(durationStr, 10);
		if (isNaN(duration) || duration <= 0) {
			return fail(400, { journalError: 'Duration must be a positive number.' });
		}

		const [existingProject] = await db
			.select({ id: project.id, status: project.status })
			.from(project)
			.where(and(eq(project.id, params.id), eq(project.userId, locals.user.id)))
			.limit(1);

		if (!existingProject) {
			error(404, 'Project not found');
		}

		assertCanEditJournal(existingProject.status);

		const [existingJournal] = await db
			.select({ id: journal.id })
			.from(journal)
			.where(and(eq(journal.id, journalId), eq(journal.projectId, params.id)))
			.limit(1);

		if (!existingJournal) {
			return fail(404, { journalError: 'Journal entry not found.' });
		}

		await db
			.update(journal)
			.set({
				durationInMinutes: duration,
				text: textVal.trim()
			})
			.where(eq(journal.id, journalId));

		return { journalSuccess: true };
	},

	submit: async ({ locals, params }) => {
		if (!locals.user) {
			error(401, 'Unauthorized');
		}

		try {
			const result = await submitProjectToAri({ projectId: params.id, userId: locals.user.id });

			return {
				success: true,
				message: `Submitted ${result.phase} review to Ari.`
			};
		} catch (err) {
			return fail(getErrorStatus(err), {
				success: false,
				message: getErrorMessage(err)
			});
		}
	},

	withdraw: async ({ locals, params }) => {
		if (!locals.user) {
			error(401, 'Unauthorized');
		}

		try {
			await withdrawProjectFromAri({ projectId: params.id, userId: locals.user.id });

			return {
				success: true,
				message: 'Project withdrawn from Ari review.'
			};
		} catch (err) {
			return fail(getErrorStatus(err), {
				success: false,
				message: getErrorMessage(err)
			});
		}
	}
};

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
