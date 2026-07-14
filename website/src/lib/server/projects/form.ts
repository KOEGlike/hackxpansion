import {
	getUserHackatimeProjects,
	getUserHackatimeProjectsWithStats,
	validateHackatimeProjectNames,
	type HackatimeProjectWithStats
} from '$lib/server/hackatime';
import { ProjectMutationError, type ProjectInput } from '$lib/server/projects/mutations';
import type { ProjectType } from '$lib/server/projects/lifecycle';

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
	return {
		title: stringFromForm(formData, 'title'),
		type: (stringFromForm(formData, 'type') as ProjectType) || undefined,
		description: stringFromForm(formData, 'description'),
		repoUrl: stringFromForm(formData, 'repoUrl'),
		demoUrl: stringFromForm(formData, 'demoUrl'),
		thumbnailUrl: stringFromForm(formData, 'thumbnailUrl'),
		hackatimeProjects: stringListFromForm(formData, 'hackatimeProjects')
	};
}

export async function getInvalidProjectHackatimeProjects(slackId: string, selected: string[]) {
	if (selected.length === 0) return [];

	try {
		const validProjects = await getUserHackatimeProjects(slackId);
		return validateHackatimeProjectNames(selected, validProjects);
	} catch {
		return [];
	}
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

export function getProjectMutationErrorStatus(err: unknown) {
	if (err instanceof ProjectMutationError) {
		return err.status;
	}

	return 500;
}

export function getErrorMessage(err: unknown) {
	if (err instanceof Error) {
		return err.message;
	}

	return 'Something went wrong.';
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
