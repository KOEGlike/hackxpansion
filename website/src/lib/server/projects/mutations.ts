import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { canEditProject } from '$lib/server/projects/lifecycle';
import { and, eq } from 'drizzle-orm';

export type ProjectInput = {
	title: string;
	description?: string | null;
	repoUrl?: string | null;
	demoUrl?: string | null;
	thumbnailUrl?: string | null;
	hackatimeProjects?: string[] | null;
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
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	status: typeof project.$inferSelect.status;
	userId: string;
	hackatimeProjects: string[] | null;
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
	const [createdProject] = await db
		.insert(project)
		.values({
			userId,
			title: values.title,
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

	return createdProject;
}

export async function editProject({ projectId, userId, input }: EditProjectOptions) {
	const [existingProject] = await db
		.select({ status: project.status })
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

	if (Object.keys(values).length === 0) {
		throw new ProjectMutationError(400, 'No project changes provided');
	}

	const [updatedProject] = await db
		.update(project)
		.set(values)
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.returning(projectReturnFields);

	if (!updatedProject) {
		throw new ProjectMutationError(404, 'Project not found');
	}

	return updatedProject;
}

const projectReturnFields = {
	id: project.id,
	title: project.title,
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
