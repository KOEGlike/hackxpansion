import { fetchWithTimeout } from '$lib/server/http';

const HACKCLUB_USERINFO_URL = 'https://auth.hackclub.com/oauth/userinfo';
const CACHET_USER_URL = 'https://cachet.dunkirk.sh/users';

type HackClubProfile = Record<string, unknown> & {
	id: string;
	name: string;
	email: string;
	emailVerified: boolean;
};

export async function fetchHackClubProfile(accessToken: string): Promise<HackClubProfile | null> {
	const response = await fetchWithTimeout(HACKCLUB_USERINFO_URL, {
		headers: { Authorization: `Bearer ${accessToken}` }
	});
	if (!response.ok) return null;

	const data = await response.json();
	if (!isRecord(data)) return null;

	const slackId = optionalString(data.slack_id);
	if (!slackId) return normalizeProfile(data);

	try {
		const slackResponse = await fetchWithTimeout(
			`${CACHET_USER_URL}/${encodeURIComponent(slackId)}`
		);
		if (!slackResponse.ok) return normalizeProfile(data);

		const slackData = await slackResponse.json();
		return normalizeProfile(isRecord(slackData) ? { ...slackData, ...data } : data);
	} catch {
		// Cachet enriches the profile but must not make authentication or refresh unavailable.
		return normalizeProfile(data);
	}
}

export function mapHackClubProfile(profile: Record<string, unknown>, checkedAt = new Date()) {
	return {
		id: optionalString(profile.sub),
		name: optionalString(profile.name),
		email: optionalString(profile.email),
		emailVerified: profile.email_verified === true,
		image: optionalString(profile.imageUrl),
		slackId: optionalString(profile.slack_id),
		verificationStatus: optionalString(profile.verification_status),
		given_name: optionalString(profile.given_name),
		yswsEligible: profile.ysws_eligible === true,
		pronouns: optionalString(profile.pronouns),
		profileCheckedAt: checkedAt
	};
}

function normalizeProfile(profile: Record<string, unknown>): HackClubProfile | null {
	const id = optionalString(profile.sub) ?? optionalString(profile.id);
	const name = optionalString(profile.name);
	const email = optionalString(profile.email);
	if (!id || !name || !email) return null;

	return {
		...profile,
		id,
		name,
		email,
		emailVerified: profile.email_verified === true
	};
}

function optionalString(value: unknown): string | undefined {
	return typeof value === 'string' ? value : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}
