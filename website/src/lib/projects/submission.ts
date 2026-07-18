import type { ProjectStatus, ProjectTier, ProjectType } from './domain';
import { getNextProjectSubmission, type ProjectReviewPhase } from './lifecycle';

export type ProjectSubmissionChangeField =
	'status' | 'description' | 'repoUrl' | 'thumbnailUrl' | 'hackatimeProjects' | 'demoUrl' | 'tier';

export type ProjectSubmissionChange = {
	field: ProjectSubmissionChangeField;
	message: string;
};

export type ProjectSubmissionReadiness = {
	canSubmit: boolean;
	phase: ProjectReviewPhase | null;
	waitingStatus: ProjectStatus | null;
	changes: ProjectSubmissionChange[];
};

export type ProjectForReadiness = {
	status: ProjectStatus;
	type: ProjectType;
	tier: ProjectTier;
	description: string | null;
	repoUrl: string | null;
	demoUrl: string | null;
	thumbnailUrl: string | null;
	hackatimeProjects: string[] | null;
};

export function getProjectSubmissionReadiness(
	project: ProjectForReadiness
): ProjectSubmissionReadiness {
	const nextSubmission = getNextProjectSubmission(project.status);

	if (!nextSubmission) {
		return {
			canSubmit: false,
			phase: null,
			waitingStatus: null,
			changes: [{ field: 'status', message: getStatusSubmissionMessage(project.status) }]
		};
	}

	const changes = getSubmissionRequirementChanges({
		...project,
		phase: nextSubmission.phase,
		requireTier: true
	});

	return {
		canSubmit: changes.length === 0,
		phase: nextSubmission.phase,
		waitingStatus: nextSubmission.waitingStatus,
		changes
	};
}

export function getSubmissionRequirementChanges({
	phase,
	type,
	tier,
	description,
	repoUrl,
	demoUrl,
	thumbnailUrl,
	hackatimeProjects,
	requireTier
}: Omit<ProjectForReadiness, 'status'> & {
	phase: ProjectReviewPhase;
	requireTier: boolean;
}): ProjectSubmissionChange[] {
	const changes: ProjectSubmissionChange[] = [];

	if (!hasText(description)) {
		changes.push({ field: 'description', message: 'Add a project description.' });
	}
	if (!hasText(repoUrl)) {
		changes.push({ field: 'repoUrl', message: 'Add a repository URL.' });
	}
	if (!hasText(thumbnailUrl)) {
		changes.push({ field: 'thumbnailUrl', message: 'Add a thumbnail URL.' });
	}
	if (!hackatimeProjects?.some((name) => hasText(name))) {
		changes.push({
			field: 'hackatimeProjects',
			message: 'Add at least one Hackatime project.'
		});
	}
	if ((phase === 'build' || type === 'app') && !hasText(demoUrl)) {
		changes.push({
			field: 'demoUrl',
			message:
				type === 'app'
					? 'Apps are software - add a demo URL before submitting to Ari.'
					: 'Add a demo URL before build review.'
		});
	}
	if (requireTier && !tier) {
		changes.push({
			field: 'tier',
			message: 'Select a tier (PRO, Advanced, or Basic) before submitting.'
		});
	}

	return changes;
}

function hasText(value: string | null | undefined) {
	return Boolean(value?.trim());
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
