import { createHmac } from 'node:crypto';
import type { ProjectReviewPhase } from '$lib/server/projects/lifecycle';

export type InboundEvidence = 'commits' | 'elapsed' | 'devlog';
export type InboundTrack = 'software' | 'hardware';

export type AriIngestPayload = {
	external_id: string;
	title: string;
	description: string;
	maker: {
		email: string;
		name: string;
		slack_id: string;
	};
	repo_url: string;
	track: InboundTrack;
	demo_url?: string;
	thumbnail_url: string;
	hackatime_projects?: string[];
	evidence: InboundEvidence[];
	journals?: {
		at: string;
		minutes: number;
		text: string;
	}[];
	meta: Record<string, string>;
};

export type ProjectForAriIngest = {
	id: string;
	title: string;
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	hackatime_projects: string[] | null;
};

export type MakerForAriIngest = {
	email: string;
	name: string;
	slackId: string;
};

export type JournalForAriIngest = {
	createdAt: Date;
	durationInMinutes: number;
};

export type BuildAriIngestPayloadOptions = {
	project: ProjectForAriIngest;
	maker: MakerForAriIngest;
	journals: JournalForAriIngest[];
	phase: ProjectReviewPhase;
	track?: InboundTrack;
};

export type SendAriIngestOptions = {
	programId: string;
	signingSecret: string;
	baseUrl?: string;
};

export type AriIngestResult = {
	status: number;
	body: string;
	alreadyQueued: boolean;
};

export class AriInboundError extends Error {
	constructor(
		readonly status: number,
		message: string,
		readonly responseBody?: string
	) {
		super(message);
		this.name = 'AriInboundError';
	}
}

export function buildAriIngestPayload({
	project,
	maker,
	journals,
	phase,
	track = 'hardware'
}: BuildAriIngestPayloadOptions): AriIngestPayload {
	const description = requiredString(project.description, 'Project description is required');
	const repoUrl = requiredString(project.repoUrl, 'Project repo URL is required');
	const thumbnailUrl = requiredString(project.thumbnailUrl, 'Project thumbnail URL is required');
	const hackatimeProjects =
		project.hackatime_projects?.filter((name) => name.trim().length > 0) ?? [];
	const ariJournals = journals
		.filter((entry) => entry.durationInMinutes > 0)
		.map((entry) => ({
			at: entry.createdAt.toISOString(),
			minutes: entry.durationInMinutes,
			text: `${formatPhase(phase)} journal entry`
		}));

	if (hackatimeProjects.length === 0) {
		throw new AriInboundError(422, 'Add at least one Hackatime project before submitting to Ari');
	}

	if (phase === 'build' && !project.demoUrl?.trim()) {
		throw new AriInboundError(422, 'Project demo URL is required for build review');
	}

	if (track === 'software' && !project.demoUrl?.trim()) {
		throw new AriInboundError(422, 'Project demo URL is required for software submissions');
	}

	return {
		external_id: project.id,
		title: project.title,
		description,
		maker: {
			email: requiredString(maker.email, 'Maker email is required'),
			name: requiredString(maker.name, 'Maker name is required'),
			slack_id: requiredString(maker.slackId, 'Maker Slack ID is required')
		},
		repo_url: repoUrl,
		track,
		...(project.demoUrl?.trim() ? { demo_url: project.demoUrl.trim() } : {}),
		thumbnail_url: thumbnailUrl,
		...(hackatimeProjects.length > 0 ? { hackatime_projects: hackatimeProjects } : {}),
		evidence: ['commits', 'elapsed', 'devlog'],
		...(ariJournals.length > 0 ? { journals: ariJournals } : {}),
		meta: {
			'Project ID': project.id,
			'Review phase': formatPhase(phase)
		}
	};
}

export async function sendAriIngest(
	payload: AriIngestPayload,
	{ programId, signingSecret, baseUrl = 'https://ari.hackclub.com' }: SendAriIngestOptions
): Promise<AriIngestResult> {
	const rawBody = JSON.stringify(payload);
	const signature = createHmac('sha256', signingSecret).update(rawBody).digest('hex');
	const url = `${baseUrl.replace(/\/$/, '')}/api/ingest/${programId}`;
	const response = await fetch(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			'X-Ari-Signature': signature
		},
		body: rawBody
	});
	const responseBody = await response.text();

	if (response.ok || response.status === 409) {
		return {
			status: response.status,
			body: responseBody,
			alreadyQueued: response.status === 409
		};
	}

	throw new AriInboundError(
		response.status,
		`Ari rejected the submission with status ${response.status}`,
		responseBody
	);
}

function requiredString(value: string | null | undefined, message: string) {
	const trimmed = value?.trim();

	if (!trimmed) {
		throw new AriInboundError(422, message);
	}

	return trimmed;
}

function formatPhase(phase: ProjectReviewPhase) {
	return phase === 'design' ? 'Design' : 'Build';
}
