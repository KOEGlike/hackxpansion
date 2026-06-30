import { env } from '$env/dynamic/private';
import {
	buildAriIngestPayload,
	sendAriIngest,
	type AriIngestResult
} from '$lib/server/ari/inbound';
import { db } from '$lib/server/db';
import { journal, project, user } from '$lib/server/db/schema';
import {
	getNextProjectSubmission,
	type ProjectReviewPhase,
	type ProjectStatus
} from '$lib/server/projects/lifecycle';
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

export class ProjectSubmissionError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'ProjectSubmissionError';
	}
}

export async function submitProjectToAri({
	projectId,
	userId
}: SubmitProjectToAriOptions): Promise<SubmitProjectToAriResult> {
	const programId = env.ARI_PROGRAM_ID;
	const signingSecret = env.ARI_IN_SECRET;

	if (!programId) {
		throw new ProjectSubmissionError(500, 'ARI_PROGRAM_ID environment variable is not set');
	}

	if (!signingSecret) {
		throw new ProjectSubmissionError(
			500,
			'ARI_IN_SECRET or ARI_SECRET environment variable is not set'
		);
	}

	const projectForSubmission = await getProjectForSubmission(projectId, userId);
	const nextSubmission = getNextProjectSubmission(projectForSubmission.status);

	if (!nextSubmission) {
		throw new ProjectSubmissionError(
			409,
			`Project cannot be submitted to Ari while its status is ${projectForSubmission.status}`
		);
	}

	const projectJournals = await db
		.select({
			createdAt: journal.createdAt,
			durationInMinutes: journal.durationInMinutes
		})
		.from(journal)
		.where(eq(journal.projectId, projectId));

	const payload = buildAriIngestPayload({
		project: projectForSubmission,
		maker: {
			email: projectForSubmission.makerEmail,
			name: projectForSubmission.makerName,
			slackId: projectForSubmission.makerSlackId
		},
		journals: projectJournals,
		phase: nextSubmission.phase
	});

	const ari = await sendAriIngest(payload, {
		programId,
		signingSecret,
		baseUrl: env.ARI_BASE_URL
	});

	const [updatedProject] = await db
		.update(project)
		.set({ status: nextSubmission.waitingStatus })
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.returning({ status: project.status });

	if (!updatedProject) {
		throw new ProjectSubmissionError(404, 'Project not found');
	}

	return {
		projectId,
		phase: nextSubmission.phase,
		status: updatedProject.status,
		ari
	};
}

async function getProjectForSubmission(projectId: string, userId: string) {
	const [row] = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			repoUrl: project.repoUrl,
			demoUrl: project.demoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			hackatime_projects: project.hackatime_projects,
			makerEmail: user.email,
			makerName: user.name,
			makerSlackId: user.slackId
		})
		.from(project)
		.innerJoin(user, eq(project.userId, user.id))
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.limit(1);

	if (!row) {
		throw new ProjectSubmissionError(404, 'Project not found');
	}

	return row;
}
