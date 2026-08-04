import { env } from '$env/dynamic/private';
import {
	AriInboundError,
	buildAriIngestPayload,
	sendAriIngest,
	sendAriWithdraw,
	type AriIngestResult,
	type AriWithdrawResult
} from '$lib/server/ari/inbound';
import { db } from '$lib/server/db';
import { journal, project, user } from '$lib/server/db/schema';
import { trackForProjectType, type ProjectReviewPhase } from '$lib/projects/lifecycle';
import {
	getProjectSubmissionReadiness,
	type ProjectSubmissionReadiness
} from '$lib/projects/submission';
import type { ProjectStatus, ProjectTier, ProjectType } from '$lib/projects/domain';
import { formatResistor } from '$lib/projects/resistors';
import { and, eq } from 'drizzle-orm';

export type SubmitProjectToAriOptions = {
	projectId: string;
	userId: string;
};

export type SubmitProjectToAriResult = {
	projectId: string;
	phase: ProjectReviewPhase;
	status: ProjectStatus;
	ari: AriIngestResult;
};

type ProjectForSubmission = {
	id: string;
	title: string;
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	status: ProjectStatus;
	type: ProjectType;
	tier: ProjectTier;
	hackatimeProjects: string[] | null;
	md0: number | null;
	md1: number | null;
	makerEmail: string;
	makerName: string;
	makerSlackId: string;
	makerYswsEligible: boolean;
};

type ClaimedSubmission = {
	project: ProjectForSubmission;
	phase: ProjectReviewPhase;
	waitingStatus: ProjectStatus;
	previousStatus: ProjectStatus;
	externalId: string;
	journals: Array<{ createdAt: Date; durationInMinutes: number; text: string }>;
};

export class ProjectSubmissionError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'ProjectSubmissionError';
	}
}

export async function canSubmit({
	projectId,
	userId
}: SubmitProjectToAriOptions): Promise<ProjectSubmissionReadiness> {
	const projectForSubmission = await getProjectForSubmission(projectId, userId);
	return getProjectSubmissionReadiness(
		projectForSubmission,
		projectForSubmission.makerYswsEligible
	);
}

export async function submitProjectToAri({
	projectId,
	userId
}: SubmitProjectToAriOptions): Promise<SubmitProjectToAriResult> {
	const config = getAriConfig();
	const claim = await claimSubmission(projectId, userId);

	try {
		const payload = buildAriIngestPayload({
			externalId: claim.externalId,
			project: {
				...claim.project,
				hackatime_projects: claim.project.hackatimeProjects
			},
			maker: {
				email: claim.project.makerEmail,
				name: claim.project.makerName,
				slackId: claim.project.makerSlackId
			},
			journals: claim.journals,
			phase: claim.phase,
			track: trackForProjectType(claim.project.type),
			extraMeta: {
				...buildResistorMeta(claim.project.type, claim.project.md0, claim.project.md1),
				...buildTierMeta(claim.project.tier)
			}
		});
		const ari = await sendAriIngest(payload, config);

		return {
			projectId,
			phase: claim.phase,
			status: claim.waitingStatus,
			ari
		};
	} catch (error) {
		if (error instanceof AriInboundError) {
			await releaseFailedSubmission(claim, userId);
			throw error;
		}

		throw new ProjectSubmissionError(
			503,
			'The submission could not be confirmed. The project remains under review to prevent a duplicate submission.'
		);
	}
}

export type WithdrawProjectFromAriResult = {
	projectId: string;
	status: ProjectStatus;
	ari: AriWithdrawResult;
};

export async function withdrawProjectFromAri({
	projectId,
	userId
}: SubmitProjectToAriOptions): Promise<WithdrawProjectFromAriResult> {
	const config = getAriConfig();
	const existingProject = await db.transaction(async (tx) => {
		const [row] = await tx
			.select({ status: project.status, activeAriExternalId: project.activeAriExternalId })
			.from(project)
			.where(and(eq(project.id, projectId), eq(project.userId, userId)))
			.limit(1)
			.for('update');

		if (!row) throw new ProjectSubmissionError(404, 'Project not found');
		if (row.status !== 'waiting_design' && row.status !== 'waiting_build') {
			throw new ProjectSubmissionError(
				409,
				'Project can only be withdrawn while it is waiting for review'
			);
		}
		const activeAriExternalId = row.activeAriExternalId;
		if (!activeAriExternalId) {
			throw new ProjectSubmissionError(409, 'The active review submission could not be identified');
		}

		return { status: row.status, activeAriExternalId };
	});

	const ari = await sendAriWithdraw(existingProject.activeAriExternalId, config);
	const revertedStatus: ProjectStatus =
		existingProject.status === 'waiting_build' ? 'approved_design' : 'not_submitted';
	const [updatedProject] = await db
		.update(project)
		.set({ status: revertedStatus, activeAriExternalId: null })
		.where(
			and(
				eq(project.id, projectId),
				eq(project.userId, userId),
				eq(project.status, existingProject.status),
				eq(project.activeAriExternalId, existingProject.activeAriExternalId)
			)
		)
		.returning({ status: project.status });

	if (!updatedProject) {
		throw new ProjectSubmissionError(409, 'The review changed while it was being withdrawn');
	}

	return { projectId, status: updatedProject.status, ari };
}

async function claimSubmission(projectId: string, userId: string): Promise<ClaimedSubmission> {
	return db.transaction(async (tx) => {
		const [projectForSubmission] = await tx
			.select(projectForSubmissionFields)
			.from(project)
			.innerJoin(user, eq(project.userId, user.id))
			.where(and(eq(project.id, projectId), eq(project.userId, userId)))
			.limit(1)
			.for('update', { of: project });

		if (!projectForSubmission) {
			throw new ProjectSubmissionError(404, 'Project not found');
		}

		const readiness = getProjectSubmissionReadiness(
			projectForSubmission,
			projectForSubmission.makerYswsEligible
		);
		if (!readiness.canSubmit || !readiness.phase || !readiness.waitingStatus) {
			throw new ProjectSubmissionError(
				readiness.changes.some((change) => change.field === 'yswsEligibility')
					? 403
					: readiness.changes.some((change) => change.field === 'status')
						? 409
						: 422,
				`Project cannot be submitted: ${readiness.changes
					.map((change) => change.message)
					.join(' ')}`
			);
		}

		const projectJournals = await tx
			.select({
				createdAt: journal.createdAt,
				durationInMinutes: journal.durationInMinutes,
				text: journal.text
			})
			.from(journal)
			.where(eq(journal.projectId, projectId));
		const externalId = projectId;

		await tx
			.update(project)
			.set({
				status: readiness.waitingStatus,
				activeAriExternalId: externalId
			})
			.where(and(eq(project.id, projectId), eq(project.userId, userId)));

		return {
			project: projectForSubmission,
			phase: readiness.phase,
			waitingStatus: readiness.waitingStatus,
			previousStatus: projectForSubmission.status,
			externalId,
			journals: projectJournals
		};
	});
}

async function releaseFailedSubmission(claim: ClaimedSubmission, userId: string) {
	await db
		.update(project)
		.set({ status: claim.previousStatus, activeAriExternalId: null })
		.where(
			and(
				eq(project.id, claim.project.id),
				eq(project.userId, userId),
				eq(project.status, claim.waitingStatus),
				eq(project.activeAriExternalId, claim.externalId)
			)
		);
}

async function getProjectForSubmission(projectId: string, userId: string) {
	const [row] = await db
		.select(projectForSubmissionFields)
		.from(project)
		.innerJoin(user, eq(project.userId, user.id))
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.limit(1);

	if (!row) throw new ProjectSubmissionError(404, 'Project not found');
	return row;
}

const projectForSubmissionFields = {
	id: project.id,
	title: project.title,
	description: project.description,
	repoUrl: project.repoUrl,
	demoUrl: project.demoUrl,
	thumbnailUrl: project.thumbnailUrl,
	status: project.status,
	type: project.type,
	tier: project.tier,
	hackatimeProjects: project.hackatime_projects,
	md0: project.md0,
	md1: project.md1,
	makerEmail: user.email,
	makerName: user.name,
	makerSlackId: user.slackId,
	makerYswsEligible: user.yswsEligible
};

function getAriConfig() {
	if (!env.ARI_PROGRAM_ID) {
		throw new ProjectSubmissionError(500, 'Project submissions are temporarily unavailable');
	}
	if (!env.ARI_IN_SECRET) {
		throw new ProjectSubmissionError(500, 'Project submissions are temporarily unavailable');
	}
	return {
		programId: env.ARI_PROGRAM_ID,
		signingSecret: env.ARI_IN_SECRET,
		baseUrl: env.ARI_BASE_URL
	};
}

function buildTierMeta(tier: ProjectTier): Record<string, string> {
	if (!tier) return {};
	const label = tier === 'pro' ? 'PRO' : tier === 'advanced' ? 'Advanced' : 'Basic';
	return { 'Project Tier': label };
}

function buildResistorMeta(
	type: ProjectType,
	md0: number | null,
	md1: number | null
): Record<string, string> {
	if (type !== 'card' || md0 == null || md1 == null) return {};
	return { 'Module ID pair': `${formatResistor(md0)}Ω / ${formatResistor(md1)}Ω` };
}
