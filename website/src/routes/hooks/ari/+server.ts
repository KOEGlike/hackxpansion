import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import {
	fromOutboundEvent,
	normalizeMinutesBreakdown,
	OutboundWebhookError,
	processOutboundRequest,
	type MinutesBreakdown,
	type OutboundBody
} from '$lib/server/ari/outbound';
import { db } from '$lib/server/db';
import { project, review, user } from '$lib/server/db/schema';
import { getApprovalCurrencyPayout, getProjectStatusAfterAriEvent } from '$lib/projects/lifecycle';
import { isUuid } from '$lib/projects/domain';
import { env } from '$env/dynamic/private';
import { and, eq, sql } from 'drizzle-orm';

export const POST: RequestHandler = async ({ request }) => {
	if (!env.ARI_OUT_SECRET) {
		error(500, 'Ari webhooks are not configured');
	}

	try {
		const { body, headers } = await processOutboundRequest(request, env.ARI_OUT_SECRET);
		const result = await db.transaction(async (tx) => {
			const [activeProject] = await tx
				.select({
					id: project.id,
					status: project.status,
					tier: project.tier,
					userId: project.userId,
					designCurrencyAwarded: project.designCurrencyAwarded,
					buildCurrencyAwarded: project.buildCurrencyAwarded,
					makerEmail: user.email,
					makerSlackId: user.slackId
				})
				.from(project)
				.innerJoin(user, eq(project.userId, user.id))
				.where(eq(project.activeAriExternalId, body.external_id))
				.limit(1)
				.for('update', { of: project });

			const associatedProjectId = activeProject?.id ?? (await findAssociatedProjectId(tx, body));
			if (!associatedProjectId) {
				throw new OutboundWebhookError(404, 'Ari delivery does not match a known project');
			}

			if (activeProject) assertMakerMatches(activeProject, body);

			const inserted = await tx
				.insert(review)
				.values({
					event: fromOutboundEvent(body.event),
					ariId: body.id,
					deliveryId: headers.delivery_id,
					projectId: associatedProjectId,
					minutesBreakdown: getMinutesBreakdown(body),
					noteToMaker: body.review.note_to_maker ?? null,
					auditNote: body.review.audit_note ?? null,
					justification: body.review.justification ?? null,
					fields: body.review.fields ?? null,
					collaborators: body.collaborators ?? null,
					fraud: body.fraud ?? null,
					reviewer: body.review.reviewer ?? null,
					rawPayload: body
				})
				.onConflictDoNothing({ target: review.deliveryId })
				.returning({ id: review.id });

			if (inserted.length === 0) return { duplicate: true as const };

			let projectStatus = null;
			let currencyAwarded = 0;
			if (activeProject) {
				const nextStatus = getProjectStatusAfterAriEvent(activeProject.status, body.event);
				if (nextStatus) {
					const payout =
						body.event === 'review.approved'
							? getApprovalCurrencyPayout(activeProject.status, activeProject.tier)
							: null;
					const payoutAlreadyAwarded =
						payout?.phase === 'design'
							? activeProject.designCurrencyAwarded
							: payout?.phase === 'build'
								? activeProject.buildCurrencyAwarded
								: false;
					const payoutUpdates =
						payout && !payoutAlreadyAwarded
							? {
									currencyPaidOut: sql`${project.currencyPaidOut} + ${payout.amount}`,
									...(payout.phase === 'design'
										? { designCurrencyAwarded: true }
										: { buildCurrencyAwarded: true })
								}
							: {};
					const [updatedProject] = await tx
						.update(project)
						.set({ status: nextStatus, ...payoutUpdates })
						.where(
							and(
								eq(project.id, activeProject.id),
								eq(project.activeAriExternalId, body.external_id)
							)
						)
						.returning({ status: project.status });
					projectStatus = updatedProject?.status ?? null;

					if (updatedProject && payout && !payoutAlreadyAwarded) {
						await tx
							.update(user)
							.set({ currency: sql`${user.currency} + ${payout.amount}` })
							.where(eq(user.id, activeProject.userId));
						currencyAwarded = payout.amount;
					}
				}
			}

			return {
				duplicate: false as const,
				id: inserted[0].id,
				projectStatus,
				currencyAwarded,
				stale: !activeProject
			};
		});

		if (result.duplicate) return json({ status: 'duplicate' });
		return json({
			status: result.stale ? 'recorded_stale' : 'ok',
			id: result.id,
			project_status: result.projectStatus,
			currency_awarded: result.currencyAwarded
		});
	} catch (err) {
		if (err instanceof OutboundWebhookError) {
			console.error(`[ari/outbound] ${err.status} ${err.message}`);
			error(err.status, err.message);
		}

		console.error('[ari/outbound] Unexpected error processing webhook', err);
		throw err;
	}
};

type Transaction = Parameters<Parameters<typeof db.transaction>[0]>[0];

async function findAssociatedProjectId(tx: Transaction, body: OutboundBody) {
	const projectId = body.external_id.split(':', 1)[0];
	if (!isUuid(projectId)) return null;

	const [row] = await tx
		.select({ id: project.id })
		.from(project)
		.where(eq(project.id, projectId))
		.limit(1);
	return row?.id ?? null;
}

function assertMakerMatches(
	activeProject: { makerEmail: string; makerSlackId: string },
	body: OutboundBody
) {
	const emailMatches = activeProject.makerEmail.toLowerCase() === body.maker.email.toLowerCase();
	const slackMatches =
		body.maker.slack_id === null || body.maker.slack_id === activeProject.makerSlackId;
	if (!emailMatches || !slackMatches) {
		throw new OutboundWebhookError(422, 'Ari delivery maker does not match the project owner');
	}
}

function getMinutesBreakdown(body: OutboundBody): MinutesBreakdown | null {
	if (body.review.minutes_breakdown) {
		return normalizeMinutesBreakdown(body.review.minutes_breakdown);
	}
	if (body.review.approved_minutes !== undefined) {
		return normalizeMinutesBreakdown({ program: body.review.approved_minutes });
	}
	if (body.review.approved_hours !== undefined) {
		return normalizeMinutesBreakdown({ program: Math.round(body.review.approved_hours * 60) });
	}
	return null;
}
