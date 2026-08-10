import { describe, expect, it } from 'vitest';
import { projectSubmissionFeedbackFromForm } from './feedback';

describe('project submission feedback', () => {
	it('requires an integer NPS score from 0 to 10 and trims optional feedback', () => {
		const formData = new FormData();
		formData.set('nps', '10');
		formData.set('whatAreWeDoingWell', '  Clear project guidance  ');

		expect(projectSubmissionFeedbackFromForm(formData)).toEqual({
			nps: 10,
			howDidYouHear: null,
			whatAreWeDoingWell: 'Clear project guidance',
			howCanWeImprove: null
		});
	});

	it.each(['', '-1', '11', '5.5'])('rejects invalid NPS value %j', (nps) => {
		const formData = new FormData();
		formData.set('nps', nps);
		expect(() => projectSubmissionFeedbackFromForm(formData)).toThrow(
			'Choose an NPS score from 0 to 10.'
		);
	});
});
