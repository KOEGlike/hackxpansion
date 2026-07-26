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
			md1: project.md1,
			md2: project.md2
		})
		.from(project)
		.where(and(eq(project.type, 'card'), isNotNull(project.md1), isNotNull(project.md2)))
		.orderBy(asc(project.md1), asc(project.md2));

	return { projects };
};
