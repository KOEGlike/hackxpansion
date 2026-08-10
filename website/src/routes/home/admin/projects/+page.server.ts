import { error } from '@sveltejs/kit';
import { desc, eq, sql } from 'drizzle-orm';
import type { PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { journal, project, review, user } from '$lib/server/db/schema';
import { requireAdmin, ShopError } from '$lib/server/shop';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) error(404, 'Page not found');

	try {
		await requireAdmin(locals.user.id);
	} catch (caught) {
		if (caught instanceof ShopError) error(caught.status, caught.message);
		throw caught;
	}

	const [projects, journalStats, reviewStats] = await Promise.all([
		db
			.select({
				id: project.id,
				title: project.title,
				description: project.description,
				thumbnailUrl: project.thumbnailUrl,
				status: project.status,
				type: project.type,
				tier: project.tier,
				currencyPaidOut: project.currencyPaidOut,
				ownerName: user.name,
				ownerEmail: user.email
			})
			.from(project)
			.innerJoin(user, eq(project.userId, user.id))
			.orderBy(desc(project.id)),
		db
			.select({
				projectId: journal.projectId,
				journalCount: sql<number>`COUNT(${journal.id})`,
				totalJournalMinutes: sql<number>`COALESCE(SUM(${journal.durationInMinutes}), 0)`
			})
			.from(journal)
			.groupBy(journal.projectId),
		db
			.select({
				projectId: review.projectId,
				reviewCount: sql<number>`COUNT(${review.id})`
			})
			.from(review)
			.groupBy(review.projectId)
	]);

	const journalsByProject = new Map(journalStats.map((stats) => [stats.projectId, stats]));
	const reviewsByProject = new Map(reviewStats.map((stats) => [stats.projectId, stats]));

	return {
		projects: projects.map((currentProject) => {
			const journals = journalsByProject.get(currentProject.id);
			const reviews = reviewsByProject.get(currentProject.id);
			return {
				...currentProject,
				journalCount: Number(journals?.journalCount ?? 0),
				totalJournalMinutes: Number(journals?.totalJournalMinutes ?? 0),
				reviewCount: Number(reviews?.reviewCount ?? 0)
			};
		})
	};
};
