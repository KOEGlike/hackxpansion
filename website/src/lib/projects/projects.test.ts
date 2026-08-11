import { describe, expect, it } from 'vitest';
import { isProjectTier, isProjectType, isWaitingForReview } from './domain';
import { isValidJournalDuration, MAX_JOURNAL_DURATION_MINUTES } from './journal';
import {
	getApprovalCurrencyPayout,
	getNextProjectSubmission,
	getProjectStatusAfterAriEvent
} from './lifecycle';
import { E24_RESISTOR_VALUES, findNextAvailableResistorPair, formatResistor } from './resistors';
import { getProjectSubmissionReadiness } from './submission';
import { sumHackatimeMinutes } from './time';
import {
	formatResistorSlug,
	getProjectProgress,
	getPublicProjectKey,
	parseResistorPairSlug
} from './explore';

const readyProject = {
	status: 'not_submitted' as const,
	type: 'card' as const,
	tier: 'basic' as const,
	description: 'A useful project',
	repoUrl: 'https://example.com/repo',
	demoUrl: null,
	thumbnailUrl: 'https://example.com/image.webp',
	hackatimeProjects: ['project-one']
};

describe('project domain validation', () => {
	it('accepts only known project types and tiers', () => {
		expect(isProjectType('card')).toBe(true);
		expect(isProjectType('other')).toBe(false);
		expect(isProjectTier('advanced')).toBe(true);
		expect(isProjectTier('none')).toBe(false);
	});

	it('identifies waiting states', () => {
		expect(isWaitingForReview('waiting_design')).toBe(true);
		expect(isWaitingForReview('approved_design')).toBe(false);
	});
});

describe('project lifecycle', () => {
	it('selects the correct next review phase', () => {
		expect(getNextProjectSubmission('not_submitted')).toEqual({
			phase: 'design',
			waitingStatus: 'waiting_design'
		});
		expect(getNextProjectSubmission('approved_design')).toEqual({
			phase: 'build',
			waitingStatus: 'waiting_build'
		});
		expect(getNextProjectSubmission('approved_build')).toBeNull();
	});

	it('applies Ari decisions only to compatible waiting states', () => {
		expect(getProjectStatusAfterAriEvent('waiting_design', 'review.approved')).toBe(
			'approved_design'
		);
		expect(getProjectStatusAfterAriEvent('not_submitted', 'review.approved')).toBeNull();
		expect(getProjectStatusAfterAriEvent('rejected_build', 'review.requeued')).toBe(
			'waiting_build'
		);
	});

	it('awards design currency by tier and one currency for builds', () => {
		expect(getApprovalCurrencyPayout('waiting_design', 'pro')).toEqual({
			phase: 'design',
			amount: 3
		});
		expect(getApprovalCurrencyPayout('waiting_design', 'advanced')?.amount).toBe(2);
		expect(getApprovalCurrencyPayout('waiting_design', 'basic')?.amount).toBe(1);
		expect(getApprovalCurrencyPayout('waiting_build', null)).toEqual({
			phase: 'build',
			amount: 1
		});
		expect(getApprovalCurrencyPayout('approved_design', 'pro')).toBeNull();
	});
});

describe('submission readiness', () => {
	it('accepts a complete card design submission', () => {
		expect(getProjectSubmissionReadiness(readyProject)).toMatchObject({
			canSubmit: true,
			phase: 'design',
			changes: []
		});
	});

	it('does not require a Hackatime project', () => {
		expect(getProjectSubmissionReadiness({ ...readyProject, hackatimeProjects: [] })).toMatchObject(
			{
				canSubmit: true,
				changes: []
			}
		);
	});

	it('reports a missing app demo only once', () => {
		const readiness = getProjectSubmissionReadiness({ ...readyProject, type: 'app' });
		expect(readiness.changes.filter((change) => change.field === 'demoUrl')).toHaveLength(1);
	});

	it('requires a demo for build review', () => {
		const readiness = getProjectSubmissionReadiness({
			...readyProject,
			status: 'approved_design'
		});
		expect(readiness.changes).toContainEqual({
			field: 'demoUrl',
			message: 'Add a demo URL before build review.'
		});
	});

	it('blocks review submission for users who are not YSWS eligible', () => {
		const readiness = getProjectSubmissionReadiness(readyProject, false);

		expect(readiness.canSubmit).toBe(false);
		expect(readiness.changes).toContainEqual({
			field: 'yswsEligibility',
			message: 'Confirm your YSWS eligibility through Hack Club Auth.'
		});
	});
});

describe('journal validation', () => {
	it('accepts bounded whole-minute values', () => {
		expect(isValidJournalDuration('1')).toBe(true);
		expect(isValidJournalDuration(String(MAX_JOURNAL_DURATION_MINUTES))).toBe(true);
	});

	it('rejects partial, scientific, and excessive values', () => {
		expect(isValidJournalDuration('12minutes')).toBe(false);
		expect(isValidJournalDuration('1e3')).toBe(false);
		expect(isValidJournalDuration(String(MAX_JOURNAL_DURATION_MINUTES + 1))).toBe(false);
	});
});

describe('project utilities', () => {
	it('formats resistor values with the multiplier as the decimal separator', () => {
		expect(formatResistor(1100)).toBe('1k1');
		expect(formatResistor(1200)).toBe('1k2');
		expect(formatResistor(10000)).toBe('10k');
	});

	it('allocates the first unused resistor pair', () => {
		const first = E24_RESISTOR_VALUES[0];
		const second = E24_RESISTOR_VALUES[1];
		expect(findNextAvailableResistorPair([{ md0: first, md1: first }])).toEqual({
			md0: first,
			md1: second
		});
	});

	it('sums only selected Hackatime projects', () => {
		expect(
			sumHackatimeMinutes(
				['one'],
				[
					{ name: 'one', totalSeconds: 90 },
					{ name: 'two', totalSeconds: 600 }
				]
			)
		).toBe(2);
	});

	it('formats and parses public resistor-pair project keys', () => {
		expect(formatResistorSlug(1500)).toBe('1k5');
		expect(parseResistorPairSlug('1k5:1k6')).toEqual({ md0: 1500, md1: 1600 });
		expect(parseResistorPairSlug('1.5k:1k6')).toBeNull();
		expect(parseResistorPairSlug('1k5:999k')).toBeNull();
		expect(getPublicProjectKey({ id: 'project-id', type: 'card', md0: 1500, md1: 1600 })).toBe(
			'1k5:1k6'
		);
	});

	it('maps project statuses to matrix intensity', () => {
		expect(getProjectProgress('not_submitted')).toBe('created');
		expect(getProjectProgress('waiting_design')).toBe('created');
		expect(getProjectProgress('approved_design')).toBe('design_approved');
		expect(getProjectProgress('waiting_build')).toBe('design_approved');
		expect(getProjectProgress('approved_build')).toBe('build_approved');
	});
});
