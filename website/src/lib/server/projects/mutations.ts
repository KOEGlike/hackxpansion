import { db } from '$lib/server/db';
import { appCard, project } from '$lib/server/db/schema';
import { canEditProject, type ProjectType } from '$lib/server/projects/lifecycle';
import { and, eq, inArray } from 'drizzle-orm';

export type ProjectInput = {
	title: string;
	type?: ProjectType;
	description?: string | null;
	repoUrl?: string | null;
	demoUrl?: string | null;
	thumbnailUrl?: string | null;
	hackatimeProjects?: string[] | null;
	cardIds?: string[] | null;
};

export type ProjectPatch = Partial<ProjectInput>;

export type CreateProjectOptions = {
	userId: string;
	input: ProjectInput;
};

export type EditProjectOptions = {
	projectId: string;
	userId: string;
	input: ProjectPatch;
};

export type ProjectMutationResult = {
	id: string;
	title: string;
	type: ProjectType;
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	status: typeof project.$inferSelect.status;
	userId: string;
	hackatimeProjects: string[] | null;
	cardIds: string[];
};

export class ProjectMutationError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'ProjectMutationError';
	}
}

export async function createProject({ userId, input }: CreateProjectOptions) {
	const values = normalizeProjectInput(input);
	const cardIds = normalizeCardIds(input.cardIds);

	if (values.type === 'app') {
		await assertCardsExist(cardIds);
	}

	const [createdProject] = await db
		.insert(project)
		.values({
			userId,
			title: values.title,
			type: values.type,
			description: values.description,
			repoUrl: values.repoUrl,
			demoUrl: values.demoUrl,
			thumbnailUrl: values.thumbnailUrl,
			hackatime_projects: values.hackatimeProjects
		})
		.returning(projectReturnFields);

	if (!createdProject) {
		throw new ProjectMutationError(500, 'Failed to create project');
	}

	const settledCardIds = await syncAppCards(createdProject.id, values.type, cardIds);

	return { ...createdProject, cardIds: settledCardIds };
}

export async function editProject({ projectId, userId, input }: EditProjectOptions) {
	const [existingProject] = await db
		.select({ status: project.status, type: project.type })
		.from(project)
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.limit(1);

	if (!existingProject) {
		throw new ProjectMutationError(404, 'Project not found');
	}

	if (!canEditProject(existingProject.status)) {
		throw new ProjectMutationError(
			409,
			'Project cannot be edited while it is waiting for Ari review'
		);
	}

	const values = normalizeProjectPatch(input);

	if (Object.keys(values).length === 0 && input.cardIds === undefined) {
		throw new ProjectMutationError(400, 'No project changes provided');
	}

	let nextType = existingProject.type;
	if (values.type !== undefined) {
		nextType = values.type;
	}

	let cardIds: string[] | null = null;
	if (input.cardIds !== undefined) {
		cardIds = normalizeCardIds(input.cardIds);
		if (nextType === 'app') {
			await assertCardsExist(cardIds);
		}
	}

	const [updatedProject] = await db
		.update(project)
		.set(values)
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.returning(projectReturnFields);

	if (!updatedProject) {
		throw new ProjectMutationError(404, 'Project not found');
	}

	const settledCardIds = await syncAppCards(updatedProject.id, updatedProject.type, cardIds);

	return { ...updatedProject, cardIds: settledCardIds };
}

const projectReturnFields = {
	id: project.id,
	title: project.title,
	type: project.type,
	description: project.description,
	repoUrl: project.repoUrl,
	demoUrl: project.demoUrl,
	thumbnailUrl: project.thumbnailUrl,
	status: project.status,
	userId: project.userId,
	hackatimeProjects: project.hackatime_projects
};

function normalizeProjectInput(input: ProjectInput) {
	return {
		title: requiredString(input.title, 'Project title is required'),
		type: normalizeProjectType(input.type),
		description: optionalString(input.description),
		repoUrl: optionalString(input.repoUrl),
		demoUrl: optionalString(input.demoUrl),
		thumbnailUrl: optionalString(input.thumbnailUrl),
		hackatimeProjects: normalizeStringArray(input.hackatimeProjects)
	};
}

function normalizeProjectPatch(input: ProjectPatch) {
	const values: Partial<typeof project.$inferInsert> = {};

	if ('title' in input) {
		values.title = requiredString(input.title, 'Project title is required');
	}

	if ('type' in input) {
		values.type = normalizeProjectType(input.type);
	}

	if ('description' in input) {
		values.description = optionalString(input.description);
	}

	if ('repoUrl' in input) {
		values.repoUrl = optionalString(input.repoUrl);
	}

	if ('demoUrl' in input) {
		values.demoUrl = optionalString(input.demoUrl);
	}

	if ('thumbnailUrl' in input) {
		values.thumbnailUrl = optionalString(input.thumbnailUrl);
	}

	if ('hackatimeProjects' in input) {
		values.hackatime_projects = normalizeStringArray(input.hackatimeProjects);
	}

	return values;
}

function normalizeProjectType(type: ProjectType | undefined): ProjectType {
	return type === 'app' ? 'app' : 'card';
}

function normalizeCardIds(cardIds: string[] | null | undefined) {
	if (!cardIds) return [];
	const normalized = [...new Set(cardIds.map((id) => id.trim()).filter(Boolean))];
	return normalized;
}

async function assertCardsExist(cardIds: string[]) {
	if (cardIds.length === 0) return;

	const cards = await db
		.select({ id: project.id, type: project.type })
		.from(project)
		.where(inArray(project.id, cardIds));

	if (cards.length !== cardIds.length) {
		const missing = cardIds.filter((id) => !cards.some((card) => card.id === id));
		throw new ProjectMutationError(422, `Unknown card dependency: ${missing.join(', ')}`);
	}

	const invalid = cards.filter((card) => card.type !== 'card');

	if (invalid.length > 0) {
		throw new ProjectMutationError(
			422,
			'App dependencies must be cards (type "card"). An app cannot depend on another app.'
		);
	}
}

async function syncAppCards(
	projectId: string,
	type: ProjectType,
	cardIds: string[] | null
): Promise<string[]> {
	if (type !== 'app') {
		await db.delete(appCard).where(eq(appCard.appId, projectId));
		return [];
	}

	if (cardIds === null) {
		const existing = await db
			.select({ cardId: appCard.cardId })
			.from(appCard)
			.where(eq(appCard.appId, projectId));
		return existing.map((row) => row.cardId);
	}

	await db.delete(appCard).where(eq(appCard.appId, projectId));

	if (cardIds.length > 0) {
		await db
			.insert(appCard)
			.values(cardIds.map((cardId) => ({ appId: projectId, cardId })))
			.onConflictDoNothing();
	}

	return cardIds;
}

function requiredString(value: string | null | undefined, message: string) {
	const trimmed = value?.trim();

	if (!trimmed) {
		throw new ProjectMutationError(422, message);
	}

	return trimmed;
}

function optionalString(value: string | null | undefined) {
	const trimmed = value?.trim();
	return trimmed ? trimmed : null;
}

function normalizeStringArray(values: string[] | null | undefined) {
	if (!values) return null;

	const normalizedValues = [...new Set(values.map((value) => value.trim()).filter(Boolean))];

	return normalizedValues.length > 0 ? normalizedValues : null;
}
