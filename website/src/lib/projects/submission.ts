import type { ProjectStatus, ProjectTier, ProjectType } from './domain';
import { getNextProjectSubmission, type ProjectReviewPhase } from './lifecycle';

export type ProjectSubmissionChangeField =
	| 'status'
	| 'yswsEligibility'
	| 'activity'
	| 'description'
	| 'repoUrl'
	| 'thumbnailUrl'
	| 'demoUrl'
	| 'tier';

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
	journalCount: number;
};

export function getProjectSubmissionReadiness(
	project: ProjectForReadiness,
	yswsEligible = true
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
	if (!yswsEligible) {
		changes.unshift({
			field: 'yswsEligibility',
			message: 'Confirm your YSWS eligibility through Hack Club Auth.'
		});
	}

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
	journalCount,
	requireTier
}: Omit<ProjectForReadiness, 'status'> & {
	phase: ProjectReviewPhase;
	requireTier: boolean;
}): ProjectSubmissionChange[] {
	const changes: ProjectSubmissionChange[] = [];
	const hasHackatimeProject = hackatimeProjects?.some((name) => name.trim().length > 0) ?? false;

	if (journalCount < 1 && !hasHackatimeProject) {
		changes.push({
			field: 'activity',
			message: 'Add at least one journal entry or Hackatime project.'
		});
	}

	if (!hasText(description)) {
		changes.push({ field: 'description', message: 'Add a project description.' });
	}
	if (!hasText(repoUrl)) {
		changes.push({ field: 'repoUrl', message: 'Add a repository URL.' });
	}
	if (!hasText(thumbnailUrl)) {
		changes.push({ field: 'thumbnailUrl', message: 'Add a thumbnail URL.' });
	}
	if ((phase === 'build' || type === 'app') && !hasText(demoUrl)) {
		changes.push({
			field: 'demoUrl',
			message:
				type === 'app'
					? 'Add a url to a video of the slint live preview to the demo URL - use #cdn'
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
