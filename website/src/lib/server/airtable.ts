import { fetchWithTimeout } from '$lib/server/http';
import type { ReviewJustification } from '$lib/server/ari/outbound';

const YSWS_BASE_ID = 'appdl599Ct1mJBIrV';
const YSWS_PROJECT_SUBMISSION_TABLE_ID = 'tblU7lrTNTaSG6bk0';

const fields = {
	ariApprovalDeliveryId: 'fld12zdbAe80iobHt',
	codeUrl: 'fldZ8lCUe3xTbJQKv',
	playableUrl: 'fldws5mWl0rCqRNOB',
	howDidYouHear: 'fldtINraIoJagxVkl',
	whatAreWeDoingWell: 'fld06k5l0hQ0J305G',
	howCanWeImprove: 'fldGz5uGOuz3xkj7z',
	firstName: 'fldgkMOj2Z7mIRQOf',
	lastName: 'fldNIZ92JkP1LDwRf',
	email: 'fldJv0Dln4PapsPEy',
	screenshot: 'fld8mmY3tnlZJMmfk',
	description: 'fldu7Fw7JOizdt47X',
	githubUsername: 'fldbGNYyTAy0SyJ4y',
	birthday: 'fldumPZ3ZeVQsXJBi',
	addressLine1: 'fldycjBo3aVe8MDN7',
	addressLine2: 'fldb3DEjlMdPKXxXt',
	city: 'fldEMkpQlc1EMKtKH',
	region: 'fldpG0RbnrL8Ek0TA',
	country: 'fldirNvUK5cbUVEiC',
	postalCode: 'flds0GzB57y15zwDB',
	overrideHours: 'fld2k6dVonmsKgDNW',
	overrideHoursJustification: 'fldrOZoVZRIcmCBfx'
} as const;

export type YswsProjectApproval = {
	ariApprovalDeliveryId: string;
	project: {
		repoUrl: string | null;
		demoUrl: string | null;
		thumbnailUrl: string | null;
		description: string | null;
	};
	maker: {
		name: string;
		givenName: string | null;
		email: string;
		githubUsername: string | null;
		birthday: string | null;
		addressLine1: string | null;
		addressLine2: string | null;
		addressCity: string | null;
		addressRegion: string | null;
		addressPostalCode: string | null;
		addressCountry: string | null;
	};
	feedback: {
		howDidYouHear: string | null;
		whatAreWeDoingWell: string | null;
		howCanWeImprove: string | null;
	} | null;
	approvedMinutes: number | null;
	overrideHoursJustification: string | null;
};

type AirtableFieldValue = string | number | Array<{ url: string }>;

export function buildYswsProjectSubmissionFields(approval: YswsProjectApproval) {
	const airtableFields: Record<string, AirtableFieldValue> = {};
	const { firstName, lastName } = splitName(approval.maker.name, approval.maker.givenName);

	airtableFields[fields.ariApprovalDeliveryId] = approval.ariApprovalDeliveryId;
	setIfPresent(airtableFields, fields.codeUrl, approval.project.repoUrl);
	setIfPresent(airtableFields, fields.playableUrl, approval.project.demoUrl);
	setIfPresent(airtableFields, fields.howDidYouHear, approval.feedback?.howDidYouHear);
	setIfPresent(airtableFields, fields.whatAreWeDoingWell, approval.feedback?.whatAreWeDoingWell);
	setIfPresent(airtableFields, fields.howCanWeImprove, approval.feedback?.howCanWeImprove);
	setIfPresent(airtableFields, fields.firstName, firstName);
	setIfPresent(airtableFields, fields.lastName, lastName);
	setIfPresent(airtableFields, fields.email, approval.maker.email);
	if (approval.project.thumbnailUrl) {
		airtableFields[fields.screenshot] = [{ url: approval.project.thumbnailUrl }];
	}
	setIfPresent(airtableFields, fields.description, approval.project.description);
	setIfPresent(airtableFields, fields.githubUsername, approval.maker.githubUsername);
	setIfPresent(airtableFields, fields.birthday, approval.maker.birthday);
	setIfPresent(airtableFields, fields.addressLine1, approval.maker.addressLine1);
	setIfPresent(airtableFields, fields.addressLine2, approval.maker.addressLine2);
	setIfPresent(airtableFields, fields.city, approval.maker.addressCity);
	setIfPresent(airtableFields, fields.region, approval.maker.addressRegion);
	setIfPresent(airtableFields, fields.country, approval.maker.addressCountry);
	setIfPresent(airtableFields, fields.postalCode, approval.maker.addressPostalCode);
	if (approval.approvedMinutes !== null) {
		airtableFields[fields.overrideHours] = approval.approvedMinutes / 60;
	}
	setIfPresent(
		airtableFields,
		fields.overrideHoursJustification,
		approval.overrideHoursJustification
	);

	return airtableFields;
}

export async function createYswsProjectSubmission(
	approval: YswsProjectApproval,
	personalAccessToken: string | undefined
) {
	if (!personalAccessToken) throw new Error('AIRTABLE_PAC is not configured');

	const response = await fetchWithTimeout(
		`https://api.airtable.com/v0/${YSWS_BASE_ID}/${YSWS_PROJECT_SUBMISSION_TABLE_ID}`,
		{
			method: 'PATCH',
			headers: {
				Authorization: `Bearer ${personalAccessToken}`,
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				performUpsert: { fieldsToMergeOn: ['Ari Approval Delivery ID'] },
				records: [{ fields: buildYswsProjectSubmissionFields(approval) }]
			})
		}
	);
	const responseBody = await response.text();
	if (!response.ok) {
		throw new Error(`Airtable project submission failed (${response.status}): ${responseBody}`);
	}

	const parsed = JSON.parse(responseBody) as { records?: Array<{ id?: unknown }> };
	const recordId = parsed.records?.[0]?.id;
	if (typeof recordId !== 'string') {
		throw new Error('Airtable project submission response did not include a record ID');
	}
	return recordId;
}

export function formatAriOverrideHoursJustification(
	justification: ReviewJustification | null | undefined,
	auditNote: string | null | undefined
) {
	return justification ? JSON.stringify(justification) : auditNote?.trim() || null;
}

function setIfPresent(
	target: Record<string, AirtableFieldValue>,
	fieldId: string,
	value: string | null | undefined
) {
	if (value) target[fieldId] = value;
}

function splitName(name: string, givenName: string | null) {
	const normalizedName = name.trim();
	const normalizedGivenName = givenName?.trim() || normalizedName.split(/\s+/, 1)[0] || '';
	const lastName = normalizedName.toLowerCase().startsWith(normalizedGivenName.toLowerCase())
		? normalizedName.slice(normalizedGivenName.length).trim()
		: normalizedName.split(/\s+/).slice(1).join(' ');
	return { firstName: normalizedGivenName, lastName };
}
