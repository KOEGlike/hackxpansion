import type { PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, user } from '$lib/server/db/schema';
import { isExploreFilter } from '$lib/projects/explore';
import { and, asc, eq, ilike, inArray } from 'drizzle-orm';

export const load: PageServerLoad = async ({ url }) => {
	const query = url.searchParams.get('q')?.trim().slice(0, 100) ?? '';
	const requestedFilter = url.searchParams.get('status') ?? 'all';
	const status = isExploreFilter(requestedFilter) ? requestedFilter : 'all';

	const statusCondition =
		status === 'not_submitted'
			? eq(project.status, 'not_submitted')
			: status === 'approved_design'
				? inArray(project.status, ['approved_design', 'waiting_build', 'rejected_build'])
				: status === 'approved_build'
					? eq(project.status, 'approved_build')
					: undefined;

	const projects = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			type: project.type,
			tier: project.tier,
			md0: project.md0,
			md1: project.md1,
			currencyPaidOut: project.currencyPaidOut,
			makerName: user.displayName,
			makerImage: user.image
		})
		.from(project)
		.innerJoin(user, eq(project.userId, user.id))
		.where(and(statusCondition, query ? ilike(project.title, `%${query}%`) : undefined))
		.orderBy(asc(project.title), asc(project.id));

	return {
		projects,
		filters: { query, status }
	};
};
