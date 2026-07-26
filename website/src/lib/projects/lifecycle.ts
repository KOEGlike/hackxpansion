import {
	isWaitingForReview,
	type ProjectStatus,
	type ProjectTier,
	type ProjectType,
	type ReviewEvent
} from './domain';

export type ProjectReviewPhase = 'design' | 'build';

export type ApprovalCurrencyPayout = {
	phase: ProjectReviewPhase;
	amount: number;
};

export const trackForProjectType = (type: ProjectType): 'hardware' | 'software' =>
	type === 'app' ? 'software' : 'hardware';

export type NextProjectSubmission = {
	phase: ProjectReviewPhase;
	waitingStatus: ProjectStatus;
};

export function canEditProject(status: ProjectStatus) {
	return !isWaitingForReview(status);
}

export function getNextProjectSubmission(status: ProjectStatus): NextProjectSubmission | null {
	switch (status) {
		case 'not_submitted':
		case 'rejected_design':
			return { phase: 'design', waitingStatus: 'waiting_design' };
		case 'approved_design':
		case 'rejected_build':
			return { phase: 'build', waitingStatus: 'waiting_build' };
		default:
			return null;
	}
}

export function getProjectStatusAfterAriEvent(
	status: ProjectStatus,
	event: ReviewEvent
): ProjectStatus | null {
	switch (event) {
		case 'review.approved':
			return status === 'waiting_design'
				? 'approved_design'
				: status === 'waiting_build'
					? 'approved_build'
					: null;
		case 'review.changes':
		case 'review.rejected':
			return status === 'waiting_design'
				? 'rejected_design'
				: status === 'waiting_build'
					? 'rejected_build'
					: null;
		case 'review.reverted':
		case 'review.requeued':
			return getWaitingStatusForCurrentPhase(status);
		case 'review.fraud':
			return null;
	}
}

export function getApprovalCurrencyPayout(
	status: ProjectStatus,
	tier: ProjectTier
): ApprovalCurrencyPayout | null {
	if (status === 'waiting_build') return { phase: 'build', amount: 1 };
	if (status !== 'waiting_design' || tier === null) return null;

	const amountByTier = { pro: 3, advanced: 2, basic: 1 } as const;
	return { phase: 'design', amount: amountByTier[tier] };
}

function getWaitingStatusForCurrentPhase(status: ProjectStatus): ProjectStatus | null {
	switch (status) {
		case 'waiting_design':
		case 'approved_design':
		case 'rejected_design':
			return 'waiting_design';
		case 'waiting_build':
		case 'approved_build':
		case 'rejected_build':
			return 'waiting_build';
		default:
			return null;
	}
}
