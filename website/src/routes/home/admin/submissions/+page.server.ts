import { error, fail } from '@sveltejs/kit';
import { and, desc, eq, inArray, notExists } from 'drizzle-orm';
import type { Actions, PageServerLoad } from './$types';
import { isUuid } from '$lib/projects/domain';
import { AdminError, requireAdmin } from '$lib/server/admin';
import { db } from '$lib/server/db';
import { project, projectSubmissionFeedback, user } from '$lib/server/db/schema';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
	} catch (caught) {
		if (caught instanceof AdminError) error(caught.status, caught.message);
		throw caught;
	}

	const submissions = await db
		.select({
			id: projectSubmissionFeedback.id,
			createdAt: projectSubmissionFeedback.createdAt,
			phase: projectSubmissionFeedback.phase,
			nps: projectSubmissionFeedback.nps,
			ariExternalId: projectSubmissionFeedback.ariExternalId,
			projectRepoUrl: projectSubmissionFeedback.projectRepoUrl,
			projectId: projectSubmissionFeedback.projectId,
			projectTitle: project.title,
			projectStatus: project.status,
			activeAriExternalId: project.activeAriExternalId,
			userId: projectSubmissionFeedback.userId,
			userName: user.displayName,
			userEmail: user.email,
			userImage: user.image
		})
		.from(projectSubmissionFeedback)
		.innerJoin(project, eq(projectSubmissionFeedback.projectId, project.id))
		.innerJoin(user, eq(projectSubmissionFeedback.userId, user.id))
		.orderBy(desc(projectSubmissionFeedback.createdAt));

	return {
		submissions: submissions.map((submission) => ({
			...submission,
			isWaitingForReview:
				(submission.projectStatus === 'waiting_design' ||
					submission.projectStatus === 'waiting_build') &&
				submission.activeAriExternalId === submission.ariExternalId
		}))
	};
};

export const actions: Actions = {
	delete: async ({ locals, request }) => {
		if (!locals.user) return fail(404, { success: false, message: 'Page not found' });
		const formData = await request.formData();
		const submissionId = formData.get('submissionId');
		if (typeof submissionId !== 'string' || !isUuid(submissionId)) {
			return fail(400, { success: false, message: 'A valid submission ID is required.' });
		}

		try {
			await requireAdmin(locals.user.id);
			const activeWaitingReview = db
				.select({ id: project.id })
				.from(project)
				.where(
					and(
						eq(project.id, projectSubmissionFeedback.projectId),
						inArray(project.status, ['waiting_design', 'waiting_build']),
						eq(project.activeAriExternalId, projectSubmissionFeedback.ariExternalId)
					)
				);
			const [deletedSubmission] = await db
				.delete(projectSubmissionFeedback)
				.where(and(eq(projectSubmissionFeedback.id, submissionId), notExists(activeWaitingReview)))
				.returning({ id: projectSubmissionFeedback.id });

			if (deletedSubmission) {
				return { success: true, message: 'Submission deleted.', submissionId };
			}

			const [existingSubmission] = await db
				.select({
					projectStatus: project.status,
					activeAriExternalId: project.activeAriExternalId,
					ariExternalId: projectSubmissionFeedback.ariExternalId
				})
				.from(projectSubmissionFeedback)
				.innerJoin(project, eq(projectSubmissionFeedback.projectId, project.id))
				.where(eq(projectSubmissionFeedback.id, submissionId))
				.limit(1);
			if (!existingSubmission) {
				return fail(404, { success: false, message: 'Submission not found.', submissionId });
			}
			return fail(409, {
				success: false,
				message: 'This submission cannot be deleted while it is waiting for ARI review.',
				submissionId
			});
		} catch (caught) {
			if (caught instanceof AdminError) {
				return fail(caught.status, {
					success: false,
					message: caught.message,
					submissionId
				});
			}
			throw caught;
		}
	}
};
