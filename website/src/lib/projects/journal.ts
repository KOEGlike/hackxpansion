export const MAX_JOURNAL_DURATION_MINUTES = 7 * 24 * 60;

export function isValidJournalDuration(value: string) {
	if (!/^\d+$/.test(value)) return false;
	const minutes = Number(value);
	return Number.isSafeInteger(minutes) && minutes >= 1 && minutes <= MAX_JOURNAL_DURATION_MINUTES;
}
