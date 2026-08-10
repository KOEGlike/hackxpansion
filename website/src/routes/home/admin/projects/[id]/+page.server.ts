import { error } from '@sveltejs/kit';
import { desc, eq } from 'drizzle-orm';
import type { PageServerLoad } from './$types';
import { isUuid } from '$lib/projects/domain';
import { db } from '$lib/server/db';
import { journal, project, review, user } from '$lib/server/db/schema';
import { requireAdmin, ShopError } from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals, params }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
	} catch (caught) {
		if (caught instanceof ShopError) error(caught.status, caught.message);
		throw caught;
	}

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
			designCurrencyAwarded: project.designCurrencyAwarded,
			designApprovedType: project.designApprovedType,
			buildCurrencyAwarded: project.buildCurrencyAwarded,
			md0: project.md0,
			md1: project.md1,
			activeAriExternalId: project.activeAriExternalId,
			hackatimeProjects: project.hackatime_projects,
			userId: project.userId,
			ownerName: user.name,
			ownerEmail: user.email,
			ownerSlackId: user.slackId,
			ownerImage: user.image,
			ownerCurrency: user.currency,
			ownerYswsEligible: user.yswsEligible,
			ownerCreatedAt: user.createdAt
		})
		.from(project)
		.innerJoin(user, eq(project.userId, user.id))
		.where(eq(project.id, params.id))
		.limit(1);

	if (!existingProject) error(404, 'Project not found');

	const [journals, reviews] = await Promise.all([
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
				ariId: review.ariId,
				deliveryId: review.deliveryId,
				approvedMinutes: review.approvedMinutes,
				minutesBreakdown: review.minutesBreakdown,
				noteToMaker: review.noteToMaker,
				auditNote: review.auditNote,
				reviewer: review.reviewer
			})
			.from(review)
			.where(eq(review.projectId, params.id))
			.orderBy(desc(review.receivedAt))
	]);

	return {
		project: existingProject,
		journals,
		reviews,
		stats: {
			journalCount: journals.length,
			totalJournalMinutes: journals.reduce((total, entry) => total + entry.durationInMinutes, 0)
		}
	};
};
