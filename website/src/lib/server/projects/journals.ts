import { db } from '$lib/server/db';
import { journal, project } from '$lib/server/db/schema';
import { canEditProject } from '$lib/projects/lifecycle';
import { isUuid } from '$lib/projects/domain';
import { isValidJournalDuration, MAX_JOURNAL_DURATION_MINUTES } from '$lib/projects/journal';
import { and, eq } from 'drizzle-orm';

export type JournalInput = {
	durationInMinutes: number;
	text: string;
};

export class JournalMutationError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'JournalMutationError';
	}
}

export function journalInputFromForm(formData: FormData): JournalInput {
	const durationValue = formData.get('durationInMinutes');
	const textValue = formData.get('text');

	if (typeof durationValue !== 'string' || !/^\d+$/.test(durationValue)) {
		throw new JournalMutationError(400, 'Duration must be a whole number of minutes.');
	}
	if (!isValidJournalDuration(durationValue)) {
		throw new JournalMutationError(
			400,
			`Duration must be between 1 and ${MAX_JOURNAL_DURATION_MINUTES} minutes.`
		);
	}

	const durationInMinutes = Number(durationValue);

	if (typeof textValue !== 'string' || !textValue.trim()) {
		throw new JournalMutationError(400, 'Journal text is required.');
	}

	return { durationInMinutes, text: textValue.trim() };
}

export async function createProjectJournal({
	projectId,
	userId,
	input
}: {
	projectId: string;
	userId: string;
	input: JournalInput;
}) {
	await db.transaction(async (tx) => {
		await lockEditableProject(tx, projectId, userId);
		await tx.insert(journal).values({ ...input, projectId });
	});
}

export async function editProjectJournal({
	projectId,
	journalId,
	userId,
	input
}: {
	projectId: string;
	journalId: string;
	userId: string;
	input: JournalInput;
}) {
	if (!isUuid(journalId)) throw new JournalMutationError(400, 'Invalid journal ID.');

	await db.transaction(async (tx) => {
		await lockEditableProject(tx, projectId, userId);
		const [updated] = await tx
			.update(journal)
			.set(input)
			.where(and(eq(journal.id, journalId), eq(journal.projectId, projectId)))
			.returning({ id: journal.id });

		if (!updated) throw new JournalMutationError(404, 'Journal entry not found.');
	});
}

type Transaction = Parameters<Parameters<typeof db.transaction>[0]>[0];

async function lockEditableProject(tx: Transaction, projectId: string, userId: string) {
	const [projectRow] = await tx
		.select({ status: project.status })
		.from(project)
		.where(and(eq(project.id, projectId), eq(project.userId, userId)))
		.limit(1)
		.for('update');

	if (!projectRow) throw new JournalMutationError(404, 'Project not found.');
	if (!canEditProject(projectRow.status)) {
		throw new JournalMutationError(
			409,
			'Cannot modify journals while the project is under review.'
		);
	}
}
