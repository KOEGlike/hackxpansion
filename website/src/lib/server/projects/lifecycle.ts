import type { Event } from '$lib/server/ari/outbound';
import type { project } from '$lib/server/db/schema';

export type ProjectStatus = typeof project.$inferSelect.status;
export type ProjectReviewPhase = 'design' | 'build';

export type NextProjectSubmission = {
	phase: ProjectReviewPhase;
	waitingStatus: ProjectStatus;
};

export function canEditProject(status: ProjectStatus) {
	return status !== 'waiting_design' && status !== 'waiting_build';
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
	event: Event
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
