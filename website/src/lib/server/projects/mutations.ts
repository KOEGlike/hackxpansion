import { db } from '$lib/server/db';
import { project } from '$lib/server/db/schema';
import { canEditProject } from '$lib/projects/lifecycle';
import {
	isProjectTier,
	isProjectType,
	type ProjectTier,
	type ProjectType
} from '$lib/projects/domain';
import { findNextAvailableResistorPair, type ModuleResistor } from '$lib/projects/resistors';
import { and, eq, isNotNull, sql } from 'drizzle-orm';

const MODULE_RESISTOR_ADVISORY_LOCK_KEY = 0x6d6470; // 'mdp'

export type ProjectInput = {
	title: string;
	type?: ProjectType;
	tier?: ProjectTier;
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
	type: ProjectType;
	tier: ProjectTier;
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	status: typeof project.$inferSelect.status;
	userId: string;
	hackatimeProjects: string[] | null;
	md0: ModuleResistor | null;
	md1: ModuleResistor | null;
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

	if (values.type === 'app') {
		const [createdProject] = await db
			.insert(project)
			.values({
				userId,
				title: values.title,
				type: values.type,
				tier: values.tier,
				description: values.description,
				repoUrl: values.repoUrl,
				demoUrl: values.demoUrl,
				thumbnailUrl: values.thumbnailUrl,
				hackatime_projects: values.hackatimeProjects,
				md0: null,
				md1: null
			})
			.returning(projectReturnFields);

		if (!createdProject) {
			throw new ProjectMutationError(500, 'Failed to create project');
		}

		return { ...createdProject };
	}

	const created = await db.transaction(async (tx) => {
		await tx.execute(sql`SELECT pg_advisory_xact_lock(${MODULE_RESISTOR_ADVISORY_LOCK_KEY})`);

		const usedPairs = await tx
			.select({ md0: project.md0, md1: project.md1 })
			.from(project)
			.where(and(eq(project.type, 'card'), isNotNull(project.md0), isNotNull(project.md1)));

		const pair = findNextAvailableResistorPair(usedPairs);

		if (!pair) {
			throw new ProjectMutationError(
				503,
				'All module resistor pairs are taken. No unique md0/md1 combination is available.'
			);
		}

		const [row] = await tx
			.insert(project)
			.values({
				userId,
				title: values.title,
				type: values.type,
				tier: values.tier,
				description: values.description,
				repoUrl: values.repoUrl,
				demoUrl: values.demoUrl,
				thumbnailUrl: values.thumbnailUrl,
				hackatime_projects: values.hackatimeProjects,
				md0: pair.md0,
				md1: pair.md1
			})
			.returning(projectReturnFields);

		if (!row) {
			throw new ProjectMutationError(500, 'Failed to create project');
		}

		return row;
	});

	return { ...created };
}

export async function editProject({ projectId, userId, input }: EditProjectOptions) {
	const values = normalizeProjectPatch(input);

	if (Object.keys(values).length === 0) {
		throw new ProjectMutationError(400, 'No project changes provided');
	}

	return db.transaction(async (tx) => {
		const [existingProject] = await tx
			.select({ status: project.status, type: project.type })
			.from(project)
			.where(and(eq(project.id, projectId), eq(project.userId, userId)))
			.limit(1)
			.for('update');

		if (!existingProject) {
			throw new ProjectMutationError(404, 'Project not found');
		}

		if (!canEditProject(existingProject.status)) {
			throw new ProjectMutationError(
				409,
				'Project cannot be edited while it is waiting for Ari review'
			);
		}

		if (values.type && values.type !== existingProject.type) {
			if (values.type === 'app') {
				values.md0 = null;
				values.md1 = null;
			} else {
				await tx.execute(sql`SELECT pg_advisory_xact_lock(${MODULE_RESISTOR_ADVISORY_LOCK_KEY})`);
				const usedPairs = await tx
					.select({ md0: project.md0, md1: project.md1 })
					.from(project)
					.where(and(eq(project.type, 'card'), isNotNull(project.md0), isNotNull(project.md1)));
				const pair = findNextAvailableResistorPair(usedPairs);

				if (!pair) {
					throw new ProjectMutationError(
						503,
						'All module resistor pairs are taken. No unique md0/md1 combination is available.'
					);
				}

				values.md0 = pair.md0;
				values.md1 = pair.md1;
			}
		}

		const [updatedProject] = await tx
			.update(project)
			.set(values)
			.where(and(eq(project.id, projectId), eq(project.userId, userId)))
			.returning(projectReturnFields);

		if (!updatedProject) {
			throw new ProjectMutationError(404, 'Project not found');
		}

		return { ...updatedProject };
	});
}

const projectReturnFields = {
	id: project.id,
	title: project.title,
	type: project.type,
	tier: project.tier,
	description: project.description,
	repoUrl: project.repoUrl,
	demoUrl: project.demoUrl,
	thumbnailUrl: project.thumbnailUrl,
	status: project.status,
	userId: project.userId,
	hackatimeProjects: project.hackatime_projects,
	md0: project.md0,
	md1: project.md1
};

function normalizeProjectInput(input: ProjectInput) {
	return {
		title: requiredString(input.title, 'Project title is required'),
		type: normalizeProjectType(input.type),
		tier: normalizeProjectTier(input.tier),
		description: optionalString(input.description),
		repoUrl: validateUrl(input.repoUrl, 'Repository URL'),
		demoUrl: validateUrl(input.demoUrl, 'Demo URL'),
		thumbnailUrl: validateUrl(input.thumbnailUrl, 'Thumbnail URL'),
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

	if ('tier' in input) {
		values.tier = normalizeProjectTier(input.tier);
	}

	if ('description' in input) {
		values.description = optionalString(input.description);
	}

	if ('repoUrl' in input) {
		values.repoUrl = validateUrl(input.repoUrl, 'Repository URL');
	}

	if ('demoUrl' in input) {
		values.demoUrl = validateUrl(input.demoUrl, 'Demo URL');
	}

	if ('thumbnailUrl' in input) {
		values.thumbnailUrl = validateUrl(input.thumbnailUrl, 'Thumbnail URL');
	}

	if ('hackatimeProjects' in input) {
		values.hackatime_projects = normalizeStringArray(input.hackatimeProjects);
	}

	return values;
}

function normalizeProjectType(type: ProjectType | undefined): ProjectType {
	if (type === undefined) return 'card';
	if (!isProjectType(type)) {
		throw new ProjectMutationError(422, 'Project type must be Card or App.');
	}
	return type;
}

function normalizeProjectTier(tier: ProjectTier | undefined): ProjectTier {
	if (tier === undefined || tier === null) return null;
	if (!isProjectTier(tier)) {
		throw new ProjectMutationError(422, 'Project tier must be PRO, Advanced, or Basic.');
	}
	return tier;
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

function validateUrl(value: string | null | undefined, field: string) {
	const trimmed = value?.trim();
	if (!trimmed) return null;

	let parsed: URL;
	try {
		parsed = new URL(trimmed);
	} catch {
		throw new ProjectMutationError(422, `${field} must be a valid URL.`);
	}

	if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
		throw new ProjectMutationError(422, `${field} must be an http or https URL.`);
	}

	return trimmed;
}

function normalizeStringArray(values: string[] | null | undefined) {
	if (!values) return null;

	const normalizedValues = [...new Set(values.map((value) => value.trim()).filter(Boolean))];

	return normalizedValues.length > 0 ? normalizedValues : null;
}
