import { formatMinutes } from './domain';

export { formatMinutes };

export function sumHackatimeMinutes(
	selectedProjects: string[] | null | undefined,
	stats: Array<{ name: string; totalSeconds: number }>
) {
	const secondsByProject = new Map(stats.map((entry) => [entry.name, entry.totalSeconds]));
	const totalSeconds = (selectedProjects ?? []).reduce(
		(total, name) => total + (secondsByProject.get(name) ?? 0),
		0
	);
	return Math.round(totalSeconds / 60);
}
