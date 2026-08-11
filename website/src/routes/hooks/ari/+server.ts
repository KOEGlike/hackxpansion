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
import { project, projectSubmissionFeedback, review, user } from '$lib/server/db/schema';
import {
	createYswsProjectSubmission,
	formatAriOverrideHoursJustification,
	type YswsProjectApproval
} from '$lib/server/airtable';
import { getApprovalCurrencyPayout, getProjectStatusAfterAriEvent } from '$lib/projects/lifecycle';
import { isUuid } from '$lib/projects/domain';
import { env } from '$env/dynamic/private';
import { and, eq, isNull, sql } from 'drizzle-orm';
import { fetchHackClubRealName } from '$lib/server/hackclub-real-name';

export const POST: RequestHandler = async ({ request }) => {
	if (!env.ARI_OUT_SECRET) {
		error(500, 'Ari webhooks are not configured');
	}

	try {
		const { body, headers } = await processOutboundRequest(request, env.ARI_OUT_SECRET);
		const minutesBreakdown = getMinutesBreakdown(body);
		const result = await db.transaction(async (tx) => {
			const [activeProject] = await tx
				.select(approvalProjectFields)
				.from(project)
				.innerJoin(user, eq(project.userId, user.id))
				.where(eq(project.activeAriExternalId, body.external_id))
				.limit(1)
				.for('update', { of: project });

			const associatedProjectId = activeProject?.id ?? (await findAssociatedProjectId(tx, body));
			if (!associatedProjectId) {
				throw new OutboundWebhookError(404, 'Ari delivery does not match a known project');
			}

			const [submissionFeedback] = await tx
				.select({
					howDidYouHear: projectSubmissionFeedback.howDidYouHear,
					whatAreWeDoingWell: projectSubmissionFeedback.whatAreWeDoingWell,
					howCanWeImprove: projectSubmissionFeedback.howCanWeImprove,
					githubUsername: projectSubmissionFeedback.githubUsername,
					birthday: projectSubmissionFeedback.birthday,
					addressLine1: projectSubmissionFeedback.addressLine1,
					addressLine2: projectSubmissionFeedback.addressLine2,
					addressCity: projectSubmissionFeedback.addressCity,
					addressRegion: projectSubmissionFeedback.addressRegion,
					addressPostalCode: projectSubmissionFeedback.addressPostalCode,
					addressCountry: projectSubmissionFeedback.addressCountry,
					projectRepoUrl: projectSubmissionFeedback.projectRepoUrl,
					projectDemoUrl: projectSubmissionFeedback.projectDemoUrl,
					projectThumbnailUrl: projectSubmissionFeedback.projectThumbnailUrl,
					projectDescription: projectSubmissionFeedback.projectDescription,
					makerEmail: projectSubmissionFeedback.makerEmail,
					makerSlackId: projectSubmissionFeedback.makerSlackId
				})
				.from(projectSubmissionFeedback)
				.where(eq(projectSubmissionFeedback.ariExternalId, body.external_id))
				.limit(1);

			if (activeProject) {
				assertMakerMatches(
					submissionFeedback
						? {
								makerEmail: submissionFeedback.makerEmail,
								makerSlackId: submissionFeedback.makerSlackId
							}
						: activeProject,
					body
				);
			}

			const inserted = await tx
				.insert(review)
				.values({
					event: fromOutboundEvent(body.event),
					ariId: body.id,
					deliveryId: headers.delivery_id,
					projectId: associatedProjectId,
					minutesBreakdown,
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
				.returning({ id: review.id, airtableRecordId: review.airtableRecordId });

			const [existingReview] = inserted.length
				? inserted
				: await tx
						.select({ id: review.id, airtableRecordId: review.airtableRecordId })
						.from(review)
						.where(eq(review.deliveryId, headers.delivery_id))
						.limit(1);
			if (!existingReview) throw new Error('Could not resolve the stored Ari review');
			const duplicate = inserted.length === 0;

			let projectStatus = null;
			let currencyAwarded = 0;
			if (!duplicate && activeProject) {
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
										? {
												designCurrencyAwarded: true,
												designApprovedType: activeProject.type
											}
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

			let airtableApproval: YswsProjectApproval | null = null;
			let airtableUserId: string | null = null;
			if (body.event === 'review.approved' && !existingReview.airtableRecordId) {
				const approvalProject =
					activeProject ??
					(
						await tx
							.select(approvalProjectFields)
							.from(project)
							.innerJoin(user, eq(project.userId, user.id))
							.where(eq(project.id, associatedProjectId))
							.limit(1)
					)[0];
				if (!approvalProject) throw new Error('Could not load the approved project');
				airtableUserId = approvalProject.userId;
				airtableApproval = {
					ariApprovalDeliveryId: headers.delivery_id,
					project: {
						repoUrl: submissionFeedback
							? submissionFeedback.projectRepoUrl
							: approvalProject.repoUrl,
						demoUrl: submissionFeedback
							? submissionFeedback.projectDemoUrl
							: approvalProject.demoUrl,
						thumbnailUrl: submissionFeedback
							? submissionFeedback.projectThumbnailUrl
							: approvalProject.thumbnailUrl,
						description: submissionFeedback
							? submissionFeedback.projectDescription
							: approvalProject.description
					},
					maker: {
						name: approvalProject.makerDisplayName,
						givenName: null,
						email: submissionFeedback ? submissionFeedback.makerEmail : approvalProject.makerEmail,
						githubUsername: submissionFeedback
							? submissionFeedback.githubUsername
							: approvalProject.makerGithubUsername,
						birthday: submissionFeedback?.birthday ?? approvalProject.makerBirthday,
						addressLine1: submissionFeedback
							? submissionFeedback.addressLine1
							: approvalProject.makerAddressLine1,
						addressLine2: submissionFeedback
							? submissionFeedback.addressLine2
							: approvalProject.makerAddressLine2,
						addressCity: submissionFeedback
							? submissionFeedback.addressCity
							: approvalProject.makerAddressCity,
						addressRegion: submissionFeedback
							? submissionFeedback.addressRegion
							: approvalProject.makerAddressRegion,
						addressPostalCode: submissionFeedback
							? submissionFeedback.addressPostalCode
							: approvalProject.makerAddressPostalCode,
						addressCountry: submissionFeedback
							? submissionFeedback.addressCountry
							: approvalProject.makerAddressCountry
					},
					feedback: submissionFeedback
						? {
								howDidYouHear: submissionFeedback.howDidYouHear,
								whatAreWeDoingWell: submissionFeedback.whatAreWeDoingWell,
								howCanWeImprove: submissionFeedback.howCanWeImprove
							}
						: null,
					approvedMinutes: sumApprovedMinutes(minutesBreakdown),
					overrideHoursJustification: formatAriOverrideHoursJustification(
						body.review.justification,
						body.review.audit_note
					)
				};
			}

			return {
				duplicate,
				id: existingReview.id,
				projectStatus,
				currencyAwarded,
				stale: !activeProject,
				airtableApproval,
				airtableUserId
			};
		});

		if (result.airtableApproval && result.airtableUserId) {
			const realName = await fetchHackClubRealName(result.airtableUserId);
			const airtableRecordId = await createYswsProjectSubmission(
				{
					...result.airtableApproval,
					maker: { ...result.airtableApproval.maker, name: realName, givenName: null }
				},
				env.AIRTABLE_PAC
			);
			await db
				.update(review)
				.set({ airtableRecordId })
				.where(and(eq(review.id, result.id), isNull(review.airtableRecordId)));
		}

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

const approvalProjectFields = {
	id: project.id,
	status: project.status,
	type: project.type,
	tier: project.tier,
	userId: project.userId,
	designCurrencyAwarded: project.designCurrencyAwarded,
	buildCurrencyAwarded: project.buildCurrencyAwarded,
	repoUrl: project.repoUrl,
	demoUrl: project.demoUrl,
	thumbnailUrl: project.thumbnailUrl,
	description: project.description,
	makerDisplayName: user.displayName,
	makerEmail: user.email,
	makerSlackId: user.slackId,
	makerGithubUsername: user.githubUsername,
	makerBirthday: user.birthday,
	makerAddressLine1: user.addressLine1,
	makerAddressLine2: user.addressLine2,
	makerAddressCity: user.addressCity,
	makerAddressRegion: user.addressRegion,
	makerAddressPostalCode: user.addressPostalCode,
	makerAddressCountry: user.addressCountry
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

function sumApprovedMinutes(breakdown: MinutesBreakdown | null) {
	if (!breakdown) return null;
	return breakdown.hackatime + breakdown.journals + breakdown.lapse + breakdown.program;
}
