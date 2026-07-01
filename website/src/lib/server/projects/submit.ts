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
	trackForProjectType,
	type ProjectReviewPhase,
	type ProjectStatus,
	type ProjectType
} from '$lib/server/projects/lifecycle';
import { listCardDependencies } from '$lib/server/projects/queries';
import { formatResistor } from '$lib/server/projects/resistors';
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

export type ProjectSubmissionChangeField =
	'status' | 'description' | 'repoUrl' | 'thumbnailUrl' | 'hackatimeProjects' | 'demoUrl';

export type ProjectSubmissionChange = {
	field: ProjectSubmissionChangeField;
	message: string;
};

export type CanSubmitProjectResult = {
	canSubmit: boolean;
	phase: ProjectReviewPhase | null;
	waitingStatus: ProjectStatus | null;
	changes: ProjectSubmissionChange[];
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
	hackatime_projects: string[] | null;
	md1: number | null;
	md2: number | null;
	makerEmail: string;
	makerName: string;
	makerSlackId: string;
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
}: SubmitProjectToAriOptions): Promise<CanSubmitProjectResult> {
	const projectForSubmission = await getProjectForSubmission(projectId, userId);
	return getProjectSubmissionReadiness(projectForSubmission);
}

export async function submitProjectToAri({
	projectId,
	userId
}: SubmitProjectToAriOptions): Promise<SubmitProjectToAriResult> {
	const programId = env.ARI_PROGRAM_ID;
	const signingSecret = env.ARI_IN_SECRET ?? env.ARI_SECRET;

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
	const readiness = getProjectSubmissionReadiness(projectForSubmission);

	if (!readiness.canSubmit || !readiness.phase || !readiness.waitingStatus) {
		throw new ProjectSubmissionError(
			getSubmissionReadinessErrorStatus(readiness),
			`Project cannot be submitted to Ari: ${formatSubmissionChanges(readiness.changes)}`
		);
	}

	const projectJournals = await db
		.select({
			createdAt: journal.createdAt,
			durationInMinutes: journal.durationInMinutes
		})
		.from(journal)
		.where(eq(journal.projectId, projectId));

	const cardDeps = projectForSubmission.type === 'app' ? await listCardDependencies(projectId) : [];

	const extraMeta: Record<string, string> = {
		...buildResistorMeta(
			projectForSubmission.type,
			projectForSubmission.md1,
			projectForSubmission.md2
		),
		...buildCardDependencyMeta(cardDeps)
	};

	const payload = buildAriIngestPayload({
		project: projectForSubmission,
		maker: {
			email: projectForSubmission.makerEmail,
			name: projectForSubmission.makerName,
			slackId: projectForSubmission.makerSlackId
		},
		journals: projectJournals,
		phase: readiness.phase,
		track: trackForProjectType(projectForSubmission.type),
		extraMeta
	});

	const ari = await sendAriIngest(payload, {
		programId,
		signingSecret,
		baseUrl: env.ARI_BASE_URL
	});

	const [updatedProject] = await db
		.update(project)
		.set({ status: readiness.waitingStatus })
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.returning({ status: project.status });

	if (!updatedProject) {
		throw new ProjectSubmissionError(404, 'Project not found');
	}

	return {
		projectId,
		phase: readiness.phase,
		status: updatedProject.status,
		ari
	};
}

function getProjectSubmissionReadiness(
	projectForSubmission: ProjectForSubmission
): CanSubmitProjectResult {
	const nextSubmission = getNextProjectSubmission(projectForSubmission.status);
	const changes: ProjectSubmissionChange[] = [];

	if (!nextSubmission) {
		changes.push({
			field: 'status',
			message: getStatusSubmissionMessage(projectForSubmission.status)
		});

		return {
			canSubmit: false,
			phase: null,
			waitingStatus: null,
			changes
		};
	}

	if (!hasText(projectForSubmission.description)) {
		changes.push({ field: 'description', message: 'Add a project description.' });
	}

	if (!hasText(projectForSubmission.repoUrl)) {
		changes.push({ field: 'repoUrl', message: 'Add a repository URL.' });
	}

	if (!hasText(projectForSubmission.thumbnailUrl)) {
		changes.push({ field: 'thumbnailUrl', message: 'Add a thumbnail URL.' });
	}

	if (!hasHackatimeProjects(projectForSubmission.hackatime_projects)) {
		changes.push({
			field: 'hackatimeProjects',
			message: 'Add at least one Hackatime project.'
		});
	}

	if (nextSubmission.phase === 'build' && !hasText(projectForSubmission.demoUrl)) {
		changes.push({ field: 'demoUrl', message: 'Add a demo URL before build review.' });
	}

	if (projectForSubmission.type === 'app' && !hasText(projectForSubmission.demoUrl)) {
		changes.push({
			field: 'demoUrl',
			message: 'Apps are software - add a demo URL before submitting to Ari.'
		});
	}

	return {
		canSubmit: changes.length === 0,
		phase: nextSubmission.phase,
		waitingStatus: nextSubmission.waitingStatus,
		changes
	};
}

async function getProjectForSubmission(
	projectId: string,
	userId: string
): Promise<ProjectForSubmission> {
	const [row] = await db
		.select({
			id: project.id,
			title: project.title,
			description: project.description,
			repoUrl: project.repoUrl,
			demoUrl: project.demoUrl,
			thumbnailUrl: project.thumbnailUrl,
			status: project.status,
			type: project.type,
			hackatime_projects: project.hackatime_projects,
			md1: project.md1,
			md2: project.md2,
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

function hasText(value: string | null | undefined) {
	return value?.trim().length ? true : false;
}

function hasHackatimeProjects(projects: string[] | null | undefined) {
	return projects?.some((name) => name.trim().length > 0) ?? false;
}

function getStatusSubmissionMessage(status: ProjectStatus) {
	switch (status) {
		case 'waiting_design':
			return 'Wait for the current design review to finish.';
		case 'waiting_build':
			return 'Wait for the current build review to finish.';
		case 'approved_build':
			return 'This project build has already been approved.';
		default:
			return `Project status must change before submitting. Current status: ${status}.`;
	}
}

function getSubmissionReadinessErrorStatus(readiness: CanSubmitProjectResult) {
	return readiness.changes.some((change) => change.field === 'status') ? 409 : 422;
}

function formatSubmissionChanges(changes: ProjectSubmissionChange[]) {
	return changes.map((change) => change.message).join(' ');
}

function buildCardDependencyMeta(
	cards: { id: string; title: string; repoUrl: string | null; status: string }[]
): Record<string, string> {
	if (cards.length === 0) return {};

	const meta: Record<string, string> = {
		'Depends on cards': String(cards.length)
	};

	for (const card of cards) {
		const label = `Card: ${card.title}`;
		const link = card.repoUrl ?? card.id;
		meta[label] = `${link} (status: ${card.status})`;
	}

	return meta;
}

function buildResistorMeta(
	type: ProjectType,
	md1: number | null,
	md2: number | null
): Record<string, string> {
	if (type !== 'card' || md1 == null || md2 == null) return {};

	return {
		'Module ID MD1': `${formatResistor(md1)}Ω`,
		'Module ID MD2': `${formatResistor(md2)}Ω`,
		'Module ID pair': `${formatResistor(md1)}Ω / ${formatResistor(md2)}Ω`
	};
}
