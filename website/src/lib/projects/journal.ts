export const MAX_JOURNAL_DURATION_MINUTES = 7 * 24 * 60;

const MARKDOWN_IMAGE_PATTERN = /!\[[^\]\n]*\]\(\s*<?https?:\/\/[^\s)>]+>?[^\n)]*\)/i;

export function isValidJournalDuration(value: string) {
	if (!/^\d+$/.test(value)) return false;
	const minutes = Number(value);
	return Number.isSafeInteger(minutes) && minutes >= 1 && minutes <= MAX_JOURNAL_DURATION_MINUTES;
}

export function hasMarkdownImage(value: string) {
	return MARKDOWN_IMAGE_PATTERN.test(value);
}
