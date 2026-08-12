import { createHmac } from 'node:crypto';
import type { ProjectReviewPhase } from '$lib/projects/lifecycle';
import { getSubmissionRequirementChanges } from '$lib/projects/submission';
import { fetchWithTimeout } from '$lib/server/http';

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
	text: string;
};

export type BuildAriIngestPayloadOptions = {
	externalId: string;
	project: ProjectForAriIngest;
	maker: MakerForAriIngest;
	journals: JournalForAriIngest[];
	phase: ProjectReviewPhase;
	track?: InboundTrack;
	extraMeta?: Record<string, string>;
};

export type SendAriIngestOptions = {
	programId: string;
	signingSecret: string;
	baseUrl?: string;
};

export type AriIngestResult = {
	status: number;
	body: string;
	duplicate: boolean;
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
	externalId,
	project,
	maker,
	journals,
	phase,
	track = 'hardware',
	extraMeta
}: BuildAriIngestPayloadOptions): AriIngestPayload {
	const hackatimeProjects =
		project.hackatime_projects?.filter((name) => name.trim().length > 0) ?? [];
	const requirementChanges = getSubmissionRequirementChanges({
		phase,
		type: track === 'software' ? 'app' : 'card',
		tier: null,
		description: project.description,
		repoUrl: project.repoUrl,
		demoUrl: project.demoUrl,
		thumbnailUrl: project.thumbnailUrl,
		hackatimeProjects,
		requireTier: false
	});

	if (requirementChanges.length > 0) {
		throw new AriInboundError(422, requirementChanges.map((change) => change.message).join(' '));
	}

	const description = project.description!.trim();
	const repoUrl = project.repoUrl!.trim();
	const thumbnailUrl = project.thumbnailUrl!.trim();
	const ariJournals = journals
		.filter((entry) => entry.durationInMinutes > 0)
		.map((entry) => ({
			at: entry.createdAt.toISOString(),
			minutes: entry.durationInMinutes,
			text: entry.text
		}));

	return {
		external_id: externalId,
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
			'Review phase': formatPhase(phase),
			...(extraMeta ?? {})
		}
	};
}

export async function sendAriIngest(
	payload: AriIngestPayload,
	{ programId, signingSecret, baseUrl = 'https://webhooks.ari.hackclub.com' }: SendAriIngestOptions
): Promise<AriIngestResult> {
	const rawBody = JSON.stringify(payload);
	const signature = createHmac('sha256', signingSecret).update(rawBody).digest('hex');
	const url = `${baseUrl.replace(/\/$/, '')}/api/ingest/${programId}`;
	const response = await fetchWithTimeout(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			'X-Ari-Signature': signature
		},
		body: rawBody
	});
	const responseBody = await response.text();

	if (response.ok) {
		return {
			status: response.status,
			body: responseBody,
			duplicate: response.status === 200
		};
	}

	throw new AriInboundError(
		response.status,
		`Ari rejected the submission with status ${response.status}`,
		responseBody
	);
}

export async function sendAriWithdraw(
	externalId: string,
	{ programId, signingSecret, baseUrl = 'https://webhooks.ari.hackclub.com' }: SendAriIngestOptions
): Promise<AriWithdrawResult> {
	const rawBody = JSON.stringify({ external_id: externalId });
	const signature = createHmac('sha256', signingSecret).update(rawBody).digest('hex');
	const url = `${baseUrl.replace(/\/$/, '')}/api/ingest/${programId}/withdraw`;
	const response = await fetchWithTimeout(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			'X-Ari-Signature': signature
		},
		body: rawBody
	});
	const responseBody = await response.text();

	if (response.ok) {
		return { status: response.status, body: responseBody };
	}

	throw new AriInboundError(
		response.status,
		`Ari rejected the withdrawal with status ${response.status}`,
		responseBody
	);
}

export type AriWithdrawResult = {
	status: number;
	body: string;
};

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
