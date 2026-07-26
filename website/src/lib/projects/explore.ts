import { isModuleResistor, type ModuleResistor } from './resistors';
import type { ProjectStatus, ProjectType } from './domain';

export const exploreFilterValues = [
	'all',
	'not_submitted',
	'approved_design',
	'approved_build'
] as const;

export type ExploreFilter = (typeof exploreFilterValues)[number];
export type ProjectProgress = 'created' | 'design_approved' | 'build_approved';

export function isExploreFilter(value: string): value is ExploreFilter {
	return exploreFilterValues.includes(value as ExploreFilter);
}

export function getProjectProgress(status: ProjectStatus): ProjectProgress {
	if (status === 'approved_build') return 'build_approved';
	if (status === 'approved_design' || status === 'waiting_build' || status === 'rejected_build') {
		return 'design_approved';
	}
	return 'created';
}

export function formatResistorSlug(ohms: ModuleResistor): string {
	const wholeKiloohms = Math.floor(ohms / 1000);
	const hundreds = (ohms % 1000) / 100;
	return hundreds === 0 ? `${wholeKiloohms}k` : `${wholeKiloohms}k${hundreds}`;
}

export function parseResistorPairSlug(value: string): {
	md0: ModuleResistor;
	md1: ModuleResistor;
} | null {
	const [md0Slug, md1Slug, extra] = value.toLowerCase().split(':');
	if (!md0Slug || !md1Slug || extra !== undefined) return null;

	const md0 = parseResistorSlug(md0Slug);
	const md1 = parseResistorSlug(md1Slug);
	return md0 && md1 ? { md0, md1 } : null;
}

export function getPublicProjectKey(project: {
	id: string;
	type: ProjectType;
	md0: number | null;
	md1: number | null;
}): string {
	if (
		project.type === 'card' &&
		project.md0 !== null &&
		project.md1 !== null &&
		isModuleResistor(project.md0) &&
		isModuleResistor(project.md1)
	) {
		return `${formatResistorSlug(project.md0)}:${formatResistorSlug(project.md1)}`;
	}
	return project.id;
}

function parseResistorSlug(value: string): ModuleResistor | null {
	const match = /^(\d+)k(\d)?$/.exec(value);
	if (!match) return null;

	const ohms = Number(match[1]) * 1000 + Number(match[2] ?? 0) * 100;
	return isModuleResistor(ohms) ? ohms : null;
}
