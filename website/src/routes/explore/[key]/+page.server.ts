import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { db } from '$lib/server/db';
import { project, user } from '$lib/server/db/schema';
import { isUuid } from '$lib/projects/domain';
import { parseResistorPairSlug } from '$lib/projects/explore';
import { and, eq } from 'drizzle-orm';

export const load: PageServerLoad = async ({ params }) => {
	const resistorPair = parseResistorPairSlug(params.key);
	const projectCondition = isUuid(params.key)
		? eq(project.id, params.key)
		: resistorPair
			? and(
					eq(project.type, 'card'),
					eq(project.md1, resistorPair.md1),
					eq(project.md2, resistorPair.md2)
				)
			: null;

	if (!projectCondition) error(404, 'Project not found');

	const [publicProject] = await db
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
			md1: project.md1,
			md2: project.md2,
			currencyPaidOut: project.currencyPaidOut,
			makerName: user.name,
			makerImage: user.image
		})
		.from(project)
		.innerJoin(user, eq(project.userId, user.id))
		.where(projectCondition)
		.limit(1);

	if (!publicProject) error(404, 'Project not found');
	return { project: publicProject };
};
