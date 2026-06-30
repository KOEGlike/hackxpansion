import { Buffer } from 'node:buffer';
import { createHmac, timingSafeEqual } from 'node:crypto';

export const eventTypeArray = [
	'approved',
	'changes',
	'rejected',
	'reverted',
	'requeued',
	'fraud'
] as const;

export type ReviewEventType = (typeof eventTypeArray)[number];

export type Event = `review.${ReviewEventType}`;

export type ReviewDecision = 'approved' | 'changes' | 'rejected';

export type OutboundHeaders = {
	signature: string;
	timestamp: string;
	delivery_id: string;
};

export type MinutesBreakdown = {
	hackatime: number;
	journals: number;
	lapse: number;
	program: number;
};

export type Reviewer = {
	email: string;
	slack_id: string | null;
};

export type FraudReview = {
	verdict: 'passed' | 'failed' | string;
	checks: {
		email: string;
		slack_id: string | null;
		trust_score: number;
		justification: string;
	}[];
};

export type ReviewField = {
	key: string;
	label: string;
	type: string;
	value: unknown;
};

export type OutboundCollaborator = {
	email: string;
	name?: string;
	slack_id?: string | null;
	hackatime_id?: string | null;
	approved_minutes?: number;
	approved_hours?: number;
	minutes_breakdown?: MinutesBreakdown;
};

export type OutboundBody = {
	event: Event;
	decision?: ReviewDecision | null;
	id: string;
	external_id: string;
	maker: {
		email: string;
		slack_id: string | null;
	};
	collaborators?: OutboundCollaborator[];
	review: {
		approved_minutes?: number;
		approved_hours?: number;
		minutes_breakdown?: MinutesBreakdown;
		note_to_maker?: string;
		audit_note?: string;
		fields?: ReviewField[];
		reviewer?: Reviewer;
	};
	fraud?: FraudReview;
};

export type ProcessedOutboundRequest = {
	headers: OutboundHeaders;
	body: OutboundBody;
	rawBody: Buffer;
};

const SIGNATURE_TOLERANCE_SECONDS = 5 * 60;

export class OutboundWebhookError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'OutboundWebhookError';
	}
}

export function fromOutboundEvent(event: Event): ReviewEventType {
	return event.replace(/^review\./, '') as ReviewEventType;
}

export function normalizeMinutesBreakdown(
	breakdown: Partial<MinutesBreakdown> | null | undefined
): MinutesBreakdown | null {
	if (!breakdown) return null;

	return {
		hackatime: breakdown.hackatime ?? 0,
		journals: breakdown.journals ?? 0,
		lapse: breakdown.lapse ?? 0,
		program: breakdown.program ?? 0
	};
}

export async function processOutboundRequest(
	request: Request,
	signingSecret: string
): Promise<ProcessedOutboundRequest> {
	const signature = request.headers.get('X-Ari-Signature');
	const timestamp = request.headers.get('X-Ari-Timestamp');
	const deliveryId =
		request.headers.get('X-Ari-Delivery-Id') ?? request.headers.get('X-Ari-Deliver-Id');

	if (!signature || !timestamp || !deliveryId) {
		throw new OutboundWebhookError(
			400,
			'X-Ari-Signature, X-Ari-Timestamp, and X-Ari-Delivery-Id headers are required'
		);
	}

	assertFreshTimestamp(timestamp);

	const rawBody = Buffer.from(await request.arrayBuffer());
	const expectedSignature = createHmac('sha256', signingSecret)
		.update(timestamp)
		.update('.')
		.update(deliveryId)
		.update('.')
		.update(rawBody)
		.digest('hex');

	if (!signaturesMatch(signature, expectedSignature)) {
		throw new OutboundWebhookError(401, 'Invalid signature');
	}

	return {
		headers: {
			signature,
			timestamp,
			delivery_id: deliveryId
		},
		body: parseOutboundBody(rawBody),
		rawBody
	};
}

function assertFreshTimestamp(timestamp: string) {
	const timestampSeconds = Number(timestamp);

	if (!Number.isInteger(timestampSeconds)) {
		throw new OutboundWebhookError(400, 'X-Ari-Timestamp must be a unix timestamp in seconds');
	}

	const nowSeconds = Math.floor(Date.now() / 1000);

	if (Math.abs(nowSeconds - timestampSeconds) > SIGNATURE_TOLERANCE_SECONDS) {
		throw new OutboundWebhookError(401, 'Stale Ari delivery timestamp');
	}
}

function signaturesMatch(providedSignature: string, expectedSignature: string) {
	const provided = Buffer.from(providedSignature.trim().toLowerCase(), 'hex');
	const expected = Buffer.from(expectedSignature, 'hex');

	if (provided.length !== expected.length) return false;

	return timingSafeEqual(provided, expected);
}

function parseOutboundBody(rawBody: Buffer): OutboundBody {
	let parsed: unknown;

	try {
		parsed = JSON.parse(rawBody.toString('utf8'));
	} catch {
		throw new OutboundWebhookError(400, 'Invalid JSON body');
	}

	if (!isRecord(parsed)) {
		throw new OutboundWebhookError(422, 'Ari payload must be a JSON object');
	}

	if (!isOutboundEvent(parsed.event)) {
		throw new OutboundWebhookError(422, 'Invalid Ari review event');
	}

	if (typeof parsed.id !== 'string' || typeof parsed.external_id !== 'string') {
		throw new OutboundWebhookError(422, 'Ari payload requires id and external_id strings');
	}

	if (!isRecord(parsed.maker) || typeof parsed.maker.email !== 'string') {
		throw new OutboundWebhookError(422, 'Ari payload requires maker.email');
	}

	if (!isRecord(parsed.review)) {
		throw new OutboundWebhookError(422, 'Ari payload requires review');
	}

	return parsed as OutboundBody;
}

function isOutboundEvent(value: unknown): value is Event {
	if (typeof value !== 'string' || !value.startsWith('review.')) return false;

	const eventType = value.replace(/^review\./, '');
	return eventTypeArray.includes(eventType as ReviewEventType);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}
