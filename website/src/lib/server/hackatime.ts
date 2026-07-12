import { env } from '$env/dynamic/private';

const HACKATIME_API_BASE = 'https://hackatime.hackclub.com/api/v1';

export type HackatimeProjectWithStats = {
	name: string;
	totalSeconds: number;
};

export class HackatimeError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'HackatimeError';
	}
}

export async function getUserHackatimeProjects(slackId: string): Promise<string[]> {
	const projects = await getUserHackatimeProjectsWithStats(slackId);
	return projects.map((p) => p.name);
}

export async function getUserHackatimeProjectsWithStats(
	slackId: string
): Promise<HackatimeProjectWithStats[]> {
	const params = new URLSearchParams();
	if (env.HACKATIME_START_DATE) params.set('start_date', env.HACKATIME_START_DATE);
	if (env.HACKATIME_END_DATE) params.set('end_date', env.HACKATIME_END_DATE);

	const query = params.toString() ? `?${params.toString()}` : '';
	const response = await fetch(`${HACKATIME_API_BASE}/users/${slackId}/projects/details${query}`);

	if (!response.ok) {
		throw new HackatimeError(
			response.status,
			`Failed to fetch Hackatime projects (status ${response.status})`
		);
	}

	const data = (await response.json()) as {
		projects?: Array<{ name: string; total_seconds?: number }>;
	};

	return (data.projects ?? []).map((p) => ({
		name: p.name,
		totalSeconds: p.total_seconds ?? 0
	}));
}

export function validateHackatimeProjectNames(submitted: string[], valid: string[]): string[] {
	return submitted.filter((name) => !valid.includes(name));
}
