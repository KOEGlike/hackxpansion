import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	buildYswsProjectSubmissionFields,
	createYswsProjectSubmission,
	formatAriOverrideHoursJustification,
	type YswsProjectApproval
} from './airtable';

const approval: YswsProjectApproval = {
	ariApprovalDeliveryId: 'delivery-1',
	project: {
		repoUrl: 'https://github.com/maker/project',
		demoUrl: 'https://project.example.com',
		thumbnailUrl: 'https://project.example.com/screenshot.png',
		description: 'A useful project.'
	},
	maker: {
		name: 'Ada Lovelace',
		givenName: 'Ada',
		email: 'ada@example.com',
		githubUsername: 'maker',
		birthday: '1815-12-10',
		addressLine1: '1 Computing Lane',
		addressLine2: null,
		addressCity: 'London',
		addressRegion: null,
		addressPostalCode: 'SW1A 1AA',
		addressCountry: 'United Kingdom'
	},
	feedback: {
		howDidYouHear: null,
		whatAreWeDoingWell: 'Fast reviews',
		howCanWeImprove: null
	},
	approvedMinutes: 90,
	overrideHoursJustification: '{"technical_features":"Custom protocol"}'
};

afterEach(() => vi.unstubAllGlobals());

describe('YSWS Project Submission Airtable export', () => {
	it('maps approval data without setting any automation fields', () => {
		const result = buildYswsProjectSubmissionFields(approval);

		expect(result).toMatchObject({
			fldnLm94WfcThZPhe: 'delivery-1',
			fldZMQYvlbUXYVHVd: 'https://github.com/maker/project',
			fldbZc8K4GdCWhBD6: 'Ada',
			fldVDeiribxpXYeSC: 'Lovelace',
			fld42yfP0pnyc6JqR: 'Fast reviews',
			fldZDzHxDOZdiq04y: '1815-12-10',
			fldsq64DaIPVrhe4e: 1.5,
			fldScjPJRcrBYgRAp: '{"technical_features":"Custom protocol"}'
		});
		expect(Object.keys(result)).not.toContain('fldiuBloMaAPjZDZN');
		expect(Object.keys(result)).not.toContain('fld90tsdUWyDSaSYq');
		expect(Object.keys(result)).not.toContain('fldtfd5xKCpxbLG4V');
		expect(Object.keys(result)).not.toContain('fld2SYcRNyUm57GjH');
	});

	it('omits the hours override when Ari did not provide approved time', () => {
		const result = buildYswsProjectSubmissionFields({ ...approval, approvedMinutes: null });
		expect(result).not.toHaveProperty('fldsq64DaIPVrhe4e');
	});

	it('uses the Ari audit note when structured justification is absent', () => {
		expect(formatAriOverrideHoursJustification(null, ' Reviewer explanation ')).toBe(
			'Reviewer explanation'
		);
		expect(
			formatAriOverrideHoursJustification({ technical_features: 'Custom protocol' }, 'Fallback')
		).toBe('{"technical_features":"Custom protocol"}');
	});

	it('posts one record to the existing YSWS Project Submission table', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValue(
				new Response(JSON.stringify({ records: [{ id: 'recAirtableRecord' }] }), { status: 200 })
			);
		vi.stubGlobal('fetch', fetchMock);

		await expect(createYswsProjectSubmission(approval, 'pat-secret')).resolves.toBe(
			'recAirtableRecord'
		);
		const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('https://api.airtable.com/v0/appdl599Ct1mJBIrV/tblRSGunS3aJP94zf');
		expect(new Headers(init.headers).get('Authorization')).toBe('Bearer pat-secret');
		expect(init.method).toBe('PATCH');
		const body = JSON.parse(String(init.body));
		expect(body.performUpsert).toEqual({
			fieldsToMergeOn: ['Ari Approval Delivery ID']
		});
		expect(body.records).toHaveLength(1);
	});
});
