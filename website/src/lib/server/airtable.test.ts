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
			fld12zdbAe80iobHt: 'delivery-1',
			fldZ8lCUe3xTbJQKv: 'https://github.com/maker/project',
			fldgkMOj2Z7mIRQOf: 'Ada',
			fldNIZ92JkP1LDwRf: 'Lovelace',
			fld06k5l0hQ0J305G: 'Fast reviews',
			fld2k6dVonmsKgDNW: 1.5,
			fldrOZoVZRIcmCBfx: '{"technical_features":"Custom protocol"}'
		});
		expect(Object.keys(result)).not.toContain('fld7pC22CeqrQzarB');
		expect(Object.keys(result)).not.toContain('fldb2Z8SvkOENIH1t');
		expect(Object.keys(result)).not.toContain('fldAkUNw8Dt6CwMLu');
		expect(Object.keys(result)).not.toContain('flddwQLjcMKNFSBFM');
	});

	it('omits the hours override when Ari did not provide approved time', () => {
		const result = buildYswsProjectSubmissionFields({ ...approval, approvedMinutes: null });
		expect(result).not.toHaveProperty('fld2k6dVonmsKgDNW');
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
		expect(url).toBe('https://api.airtable.com/v0/appdl599Ct1mJBIrV/tblU7lrTNTaSG6bk0');
		expect(new Headers(init.headers).get('Authorization')).toBe('Bearer pat-secret');
		expect(init.method).toBe('PATCH');
		const body = JSON.parse(String(init.body));
		expect(body.performUpsert).toEqual({
			fieldsToMergeOn: ['Ari Approval Delivery ID']
		});
		expect(body.records).toHaveLength(1);
	});
});
