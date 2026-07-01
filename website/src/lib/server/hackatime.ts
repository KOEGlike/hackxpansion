const HACKATIME_BASE_URL = 'https://hackatime.hackclub.com/api/v1';

export class HackatimeError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'HackatimeError';
	}
}

export async function listUserHackatimeProjects(slackId: string): Promise<string[]> {
	const trimmed = slackId.trim();
	if (!trimmed) return [];

	const url = `${HACKATIME_BASE_URL}/users/${encodeURIComponent(trimmed)}/projects`;
	let response: Response;
	try {
		response = await fetch(url, { headers: { Accept: 'application/json' } });
	} catch (err) {
		throw new HackatimeError(
			502,
			`Could not reach Hackatime to verify your projects: ${(err as Error).message}`
		);
	}

	if (response.status === 404) {
		throw new HackatimeError(
			404,
			`No Hackatime profile found for Slack ID "${trimmed}". Ensure Hackatime is linked to your Hack Club account.`
		);
	}

	if (!response.ok) {
		throw new HackatimeError(
			502,
			`Hackatime returned status ${response.status} while listing your projects.`
		);
	}

	const body = (await response.json()) as { projects?: string[] };
	const projects = Array.isArray(body.projects) ? body.projects : [];

	return projects
		.map((name) => (typeof name === 'string' ? name.trim() : ''))
		.filter((name) => name.length > 0);
}

export async function safeListUserHackatimeProjects(slackId: string): Promise<string[]> {
	try {
		return await listUserHackatimeProjects(slackId);
	} catch {
		return [];
	}
}
