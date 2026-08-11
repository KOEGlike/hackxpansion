import { fetchWithTimeout } from '$lib/server/http';
import type { ReviewJustification } from '$lib/server/ari/outbound';

const YSWS_BASE_ID = 'appdl599Ct1mJBIrV';
const YSWS_PROJECT_SUBMISSION_TABLE_ID = 'tblRSGunS3aJP94zf';

const fields = {
	ariApprovalDeliveryId: 'fldnLm94WfcThZPhe',
	codeUrl: 'fldZMQYvlbUXYVHVd',
	playableUrl: 'fldh32ZKusdegrNnP',
	howDidYouHear: 'fldCSc0z6y7xYPFqO',
	whatAreWeDoingWell: 'fld42yfP0pnyc6JqR',
	howCanWeImprove: 'fldJmKi07TjsZDGxm',
	firstName: 'fldbZc8K4GdCWhBD6',
	lastName: 'fldVDeiribxpXYeSC',
	email: 'flda2DAyT7FVCEjtL',
	screenshot: 'fldMLTCto2rm172cn',
	description: 'fldCHa09hsRg4vrff',
	githubUsername: 'fldOzznkZN4s0J38X',
	birthday: 'fldZDzHxDOZdiq04y',
	addressLine1: 'fldjpIXY9mLnuwfer',
	addressLine2: 'fldgXGgX61VSCd0W3',
	city: 'fldEiMDCo4bOp5mw5',
	region: 'fldMqWBcBuDaHGb42',
	country: 'fldWG0CPuWoSiszSy',
	postalCode: 'fld6vnS8HvtcUsbet',
	overrideHours: 'fldsq64DaIPVrhe4e',
	hackatimeProjects: 'fldn65GYnm7Q8mfNE',
	hackatimeUserId: 'fldVpkqS5o87dqbFq',
	lapseLinks: 'fldjALcVM1u7150Dq',
	technicalFeatures: 'fld0HvqTvcI429CFa',
	deflationReason: 'fldxnLY7qf46rqQYS',
	additionalJustification: 'fldp3XWKJjES72fWM'
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
	justification: ReviewJustification | null;
	auditNote: string | null;
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
		fields.hackatimeProjects,
		approval.justification?.hackatime_projects
	);
	setIfPresent(airtableFields, fields.hackatimeUserId, approval.justification?.hackatime_user_id);
	setIfPresent(airtableFields, fields.lapseLinks, approval.justification?.lapse_links);
	setIfPresent(
		airtableFields,
		fields.technicalFeatures,
		approval.justification?.technical_features
	);
	setIfPresent(airtableFields, fields.deflationReason, approval.justification?.deflation_reason);
	if (!Object.values(approval.justification ?? {}).some((value) => value?.trim())) {
		setIfPresent(airtableFields, fields.additionalJustification, approval.auditNote?.trim());
	}

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
