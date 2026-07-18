import {
	getUserHackatimeProjects,
	getUserHackatimeProjectsWithStats,
	validateHackatimeProjectNames,
	type HackatimeProjectWithStats
} from '$lib/server/hackatime';
import { ProjectMutationError, type ProjectInput } from '$lib/server/projects/mutations';
import { isProjectTier, isProjectType } from '$lib/projects/domain';

export type ProjectFormHackatimeData = {
	hackatimeProjects: HackatimeProjectWithStats[];
	hackatimeError: string | null;
};

export async function loadProjectFormHackatimeProjects(
	slackId: string,
	errorMessage: string
): Promise<ProjectFormHackatimeData> {
	try {
		return {
			hackatimeProjects: await getUserHackatimeProjectsWithStats(slackId),
			hackatimeError: null
		};
	} catch {
		return {
			hackatimeProjects: [],
			hackatimeError: errorMessage
		};
	}
}

export function projectInputFromForm(formData: FormData): ProjectInput {
	const type = stringFromForm(formData, 'type');
	const tier = stringFromForm(formData, 'tier');

	if (!isProjectType(type)) {
		throw new ProjectMutationError(422, 'Project type must be Card or App.');
	}
	if (tier && !isProjectTier(tier)) {
		throw new ProjectMutationError(422, 'Project tier must be PRO, Advanced, or Basic.');
	}
	const normalizedTier = tier && isProjectTier(tier) ? tier : null;

	return {
		title: stringFromForm(formData, 'title'),
		type,
		tier: normalizedTier,
		description: stringFromForm(formData, 'description'),
		repoUrl: stringFromForm(formData, 'repoUrl'),
		demoUrl: stringFromForm(formData, 'demoUrl'),
		thumbnailUrl: stringFromForm(formData, 'thumbnailUrl'),
		hackatimeProjects: stringListFromForm(formData, 'hackatimeProjects')
	};
}

export async function getInvalidProjectHackatimeProjects(slackId: string, selected: string[]) {
	if (selected.length === 0) return [];

	const validProjects = await getUserHackatimeProjects(slackId);
	return validateHackatimeProjectNames(selected, validProjects);
}

export function formValuesToObject(formData: FormData): Record<string, string | string[]> {
	const values: Record<string, string | string[]> = {};
	for (const [key, value] of formData.entries()) {
		const strValue = String(value);
		if (key in values) {
			const existing = values[key];
			if (Array.isArray(existing)) {
				existing.push(strValue);
			} else {
				values[key] = [existing, strValue];
			}
		} else {
			values[key] = strValue;
		}
	}
	return values;
}

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}

function stringListFromForm(formData: FormData, key: string) {
	return formData
		.getAll(key)
		.map((value) => (typeof value === 'string' ? value : ''))
		.map((value) => value.trim())
		.filter(Boolean);
}
