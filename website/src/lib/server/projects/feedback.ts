export type ProjectSubmissionFeedbackInput = {
	nps: number;
	howDidYouHear: string | null;
	whatAreWeDoingWell: string | null;
	howCanWeImprove: string | null;
};

export type ProjectSubmissionFeedbackFormValues = {
	nps: string;
	howDidYouHear: string;
	whatAreWeDoingWell: string;
	howCanWeImprove: string;
};

export class ProjectSubmissionFeedbackError extends Error {
	constructor(message: string) {
		super(message);
		this.name = 'ProjectSubmissionFeedbackError';
	}
}

export function projectSubmissionFeedbackFormValues(
	formData: FormData
): ProjectSubmissionFeedbackFormValues {
	return {
		nps: stringFromForm(formData, 'nps'),
		howDidYouHear: stringFromForm(formData, 'howDidYouHear'),
		whatAreWeDoingWell: stringFromForm(formData, 'whatAreWeDoingWell'),
		howCanWeImprove: stringFromForm(formData, 'howCanWeImprove')
	};
}

export function projectSubmissionFeedbackFromForm(
	formData: FormData
): ProjectSubmissionFeedbackInput {
	const values = projectSubmissionFeedbackFormValues(formData);
	if (!/^(?:[0-9]|10)$/.test(values.nps)) {
		throw new ProjectSubmissionFeedbackError('Choose an NPS score from 0 to 10.');
	}

	return {
		nps: Number(values.nps),
		howDidYouHear: optionalFeedback(values.howDidYouHear),
		whatAreWeDoingWell: optionalFeedback(values.whatAreWeDoingWell),
		howCanWeImprove: optionalFeedback(values.howCanWeImprove)
	};
}

function optionalFeedback(value: string) {
	const normalized = value.trim();
	if (normalized.length > 2_000) {
		throw new ProjectSubmissionFeedbackError(
			'Each optional feedback response must be 2,000 characters or fewer.'
		);
	}
	return normalized || null;
}

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}
