import type { PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { and, asc, eq, isNotNull } from 'drizzle-orm';

export const load: PageServerLoad = async () => {
	const projects = await db
		.select({
			id: project.id,
			title: project.title,
			status: project.status,
			type: project.type,
			md0: project.md0,
			md1: project.md1
		})
		.from(project)
		.where(and(eq(project.type, 'card'), isNotNull(project.md0), isNotNull(project.md1)))
		.orderBy(asc(project.md0), asc(project.md1));

	return { projects };
};
