import { Buffer } from 'node:buffer';
import { createHmac, timingSafeEqual } from 'node:crypto';
import {
	reviewEventTypeValues,
	type ReviewEvent,
	type ReviewEventType
} from '$lib/projects/domain';

export const eventTypeArray = reviewEventTypeValues;
export type Event = ReviewEvent;

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
const MAX_WEBHOOK_BODY_BYTES = 1_000_000;
const MAX_MINUTES_VALUE = 10_000_000;

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

	return Object.fromEntries(
		(['hackatime', 'journals', 'lapse', 'program'] as const).map((key) => [
			key,
			validMinutes(breakdown[key], `review.minutes_breakdown.${key}`)
		])
	) as MinutesBreakdown;
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

	const rawBody = await readLimitedBody(request);
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

	if (parsed.review.minutes_breakdown !== undefined) {
		if (!isRecord(parsed.review.minutes_breakdown)) {
			throw new OutboundWebhookError(422, 'review.minutes_breakdown must be an object');
		}
		normalizeMinutesBreakdown(parsed.review.minutes_breakdown);
	}

	if (parsed.review.approved_minutes !== undefined) {
		validMinutes(parsed.review.approved_minutes, 'review.approved_minutes');
	}

	if (parsed.review.approved_hours !== undefined) {
		const hours = parsed.review.approved_hours;
		if (typeof hours !== 'number' || !Number.isFinite(hours) || hours < 0) {
			throw new OutboundWebhookError(422, 'review.approved_hours must be a non-negative number');
		}
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

function validMinutes(value: unknown, field: string) {
	const minutes = value ?? 0;
	if (
		!Number.isSafeInteger(minutes) ||
		Number(minutes) < 0 ||
		Number(minutes) > MAX_MINUTES_VALUE
	) {
		throw new OutboundWebhookError(
			422,
			`${field} must be a non-negative integer no greater than ${MAX_MINUTES_VALUE}`
		);
	}
	return Number(minutes);
}

async function readLimitedBody(request: Request) {
	const contentLength = Number(request.headers.get('content-length'));
	if (Number.isFinite(contentLength) && contentLength > MAX_WEBHOOK_BODY_BYTES) {
		throw new OutboundWebhookError(413, 'Ari payload is too large');
	}

	if (!request.body) return Buffer.alloc(0);

	const reader = request.body.getReader();
	const chunks: Uint8Array[] = [];
	let totalBytes = 0;

	while (true) {
		const { done, value } = await reader.read();
		if (done) break;
		totalBytes += value.byteLength;
		if (totalBytes > MAX_WEBHOOK_BODY_BYTES) {
			await reader.cancel();
			throw new OutboundWebhookError(413, 'Ari payload is too large');
		}
		chunks.push(value);
	}

	return Buffer.concat(chunks, totalBytes);
}
