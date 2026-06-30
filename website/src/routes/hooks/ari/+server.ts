import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import {
	fromOutboundEvent,
	normalizeMinutesBreakdown,
	OutboundWebhookError,
	processOutboundRequest
} from '$lib/server/ari/outbound';
import { db } from '$lib/server/db';
import { project, review } from '$lib/server/db/schema';
import { env } from '$env/dynamic/private';
import { eq } from 'drizzle-orm';

const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export const POST: RequestHandler = async ({ request }) => {
	if (!env.ARI_OUT_SECRET) {
		error(500, 'ARI_OUT_SECRET environment variable is not set');
	}

	try {
		const { body, headers } = await processOutboundRequest(request, env.ARI_OUT_SECRET);
		const projectId = await findProjectId(body.external_id);

		const inserted = await db
			.insert(review)
			.values({
				event: fromOutboundEvent(body.event),
				ariId: body.id,
				deliveryId: headers.delivery_id,
				projectId,
				minutesBreakdown: normalizeMinutesBreakdown(body.review.minutes_breakdown),
				noteToMaker: body.review.note_to_maker ?? null,
				auditNote: body.review.audit_note ?? null,
				fields: body.review.fields ?? null,
				collaborators: body.collaborators ?? null,
				fraud: body.fraud ?? null,
				reviewer: body.review.reviewer ?? null,
				rawPayload: body
			})
			.onConflictDoNothing({ target: review.deliveryId })
			.returning({ id: review.id });

		if (inserted.length === 0) {
			return json({ status: 'duplicate' });
		}

		return json({ status: 'ok', id: inserted[0].id });
	} catch (err) {
		if (err instanceof OutboundWebhookError) {
			error(err.status, err.message);
		}

		throw err;
	}
};

async function findProjectId(externalId: string) {
	if (!UUID_REGEX.test(externalId)) return null;

	const [row] = await db
		.select({ id: project.id })
		.from(project)
		.where(eq(project.id, externalId))
		.limit(1);

	return row?.id ?? null;
}
