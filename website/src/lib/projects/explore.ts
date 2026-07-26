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
	md1: ModuleResistor;
	md2: ModuleResistor;
} | null {
	const [md1Slug, md2Slug, extra] = value.toLowerCase().split(':');
	if (!md1Slug || !md2Slug || extra !== undefined) return null;

	const md1 = parseResistorSlug(md1Slug);
	const md2 = parseResistorSlug(md2Slug);
	return md1 && md2 ? { md1, md2 } : null;
}

export function getPublicProjectKey(project: {
	id: string;
	type: ProjectType;
	md1: number | null;
	md2: number | null;
}): string {
	if (
		project.type === 'card' &&
		project.md1 !== null &&
		project.md2 !== null &&
		isModuleResistor(project.md1) &&
		isModuleResistor(project.md2)
	) {
		return `${formatResistorSlug(project.md1)}:${formatResistorSlug(project.md2)}`;
	}
	return project.id;
}

function parseResistorSlug(value: string): ModuleResistor | null {
	const match = /^(\d+)k(\d)?$/.exec(value);
	if (!match) return null;

	const ohms = Number(match[1]) * 1000 + Number(match[2] ?? 0) * 100;
	return isModuleResistor(ohms) ? ohms : null;
}
