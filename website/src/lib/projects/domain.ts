export const projectStatusValues = [
	'not_submitted',
	'waiting_design',
	'rejected_design',
	'approved_design',
	'waiting_build',
	'rejected_build',
	'approved_build'
] as const;

export const projectTypeValues = ['card', 'app'] as const;
export const projectTierValues = ['pro', 'advanced', 'basic'] as const;
export const reviewEventTypeValues = [
	'approved',
	'changes',
	'rejected',
	'reverted',
	'requeued',
	'fraud'
] as const;

export type ProjectStatus = (typeof projectStatusValues)[number];
export type ProjectType = (typeof projectTypeValues)[number];
export type ProjectTier = (typeof projectTierValues)[number] | null;
export type ReviewEventType = (typeof reviewEventTypeValues)[number];
export type ReviewEvent = `review.${ReviewEventType}`;

export function isProjectType(value: unknown): value is ProjectType {
	return typeof value === 'string' && projectTypeValues.includes(value as ProjectType);
}

export function isProjectTier(value: unknown): value is Exclude<ProjectTier, null> {
	return (
		typeof value === 'string' && projectTierValues.includes(value as Exclude<ProjectTier, null>)
	);
}

export function isWaitingForReview(status: ProjectStatus) {
	return status === 'waiting_design' || status === 'waiting_build';
}

export function formatMinutes(minutes: number): string {
	const hours = Math.floor(minutes / 60);
	const remainingMinutes = minutes % 60;
	if (hours === 0) return `${remainingMinutes}m`;
	if (remainingMinutes === 0) return `${hours}h`;
	return `${hours}h ${remainingMinutes}m`;
}

export function isUuid(value: string) {
	return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}
