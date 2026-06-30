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
import { getProjectStatusAfterAriEvent } from '$lib/server/projects/lifecycle';
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

		const result = await db.transaction(async (tx) => {
			const inserted = await tx
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
				return { duplicate: true as const };
			}

			let projectStatus = null;

			if (projectId) {
				const [projectRow] = await tx
					.select({ status: project.status })
					.from(project)
					.where(eq(project.id, projectId))
					.limit(1);
				const nextStatus = projectRow
					? getProjectStatusAfterAriEvent(projectRow.status, body.event)
					: null;

				if (nextStatus) {
					const [updatedProject] = await tx
						.update(project)
						.set({ status: nextStatus })
						.where(eq(project.id, projectId))
						.returning({ status: project.status });

					projectStatus = updatedProject?.status ?? null;
				}
			}

			return { duplicate: false as const, id: inserted[0].id, projectStatus };
		});

		if (result.duplicate) {
			return json({ status: 'duplicate' });
		}

		return json({ status: 'ok', id: result.id, project_status: result.projectStatus });
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
