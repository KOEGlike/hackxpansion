import { db } from '$lib/server/db';
import { appCard, project } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';

export type SelectableCard = {
	id: string;
	title: string;
	repoUrl: string | null;
	thumbnailUrl: string | null;
	status: typeof project.$inferSelect.status;
	userId: string;
};

export type CardDependency = {
	id: string;
	title: string;
	repoUrl: string | null;
	thumbnailUrl: string | null;
	status: typeof project.$inferSelect.status;
};

export async function listSelectableCards(): Promise<SelectableCard[]> {
	const rows = await db
		.select({
			id: project.id,
			title: project.title,
			repoUrl: project.repoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			userId: project.userId
		})
		.from(project)
		.where(eq(project.type, 'card'));

	return rows;
}

export async function listCardDependencies(appId: string): Promise<CardDependency[]> {
	const rows = await db
		.select({
			id: project.id,
			title: project.title,
			repoUrl: project.repoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status
		})
		.from(appCard)
		.innerJoin(project, eq(appCard.cardId, project.id))
		.where(eq(appCard.appId, appId));

	return rows;
}

export async function listCardDependencyIds(appId: string): Promise<string[]> {
	const rows = await db
		.select({ cardId: appCard.cardId })
		.from(appCard)
		.where(eq(appCard.appId, appId));
	return rows.map((row) => row.cardId);
}
