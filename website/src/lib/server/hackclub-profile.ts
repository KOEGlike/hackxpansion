import { fetchWithTimeout } from '$lib/server/http';

const HACKCLUB_USERINFO_URL = 'https://auth.hackclub.com/oauth/userinfo';
const SLACK_USERINFO_URL = 'https://slack.com/api/users.info';

type HackClubProfile = Record<string, unknown> & {
	id: string;
	email: string;
	emailVerified: boolean;
};

async function fetchHackClubUserInfo(accessToken: string) {
	const response = await fetchWithTimeout(HACKCLUB_USERINFO_URL, {
		headers: { Authorization: `Bearer ${accessToken}` }
	});
	if (!response.ok) return null;

	const data = await response.json();
	if (!isRecord(data)) return null;

	return data;
}

export async function fetchHackClubRealName(accessToken: string) {
	const data = await fetchHackClubUserInfo(accessToken);
	const realName = data && optionalString(data.name)?.trim();
	if (!realName) throw new Error('Hack Club profile did not include a real name');
	return realName;
}

export async function fetchHackClubProfile(
	accessToken: string,
	slackBotToken: string | undefined
): Promise<HackClubProfile | null> {
	const data = await fetchHackClubUserInfo(accessToken);
	const profile = data && normalizeProfile(data);
	if (!profile) return null;

	const slackId = optionalString(profile.slack_id);
	if (!slackId) throw new Error('Hack Club profile did not include a Slack ID');
	if (!slackBotToken) throw new Error('SLACK_BOT_TOKEN is not configured');

	const url = new URL(SLACK_USERINFO_URL);
	url.searchParams.set('user', slackId);
	const slackResponse = await fetchWithTimeout(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${slackBotToken}`
		},
		body: JSON.stringify({ user: slackId })
	});
	if (!slackResponse.ok) throw new Error(`Slack users.info failed (${slackResponse.status})`);

	const slackData = await slackResponse.json();
	if (!isRecord(slackData) || slackData.ok !== true || !isRecord(slackData.user)) {
		throw new Error('Slack users.info returned an invalid response');
	}
	const slackProfile = slackData.user.profile;
	if (!isRecord(slackProfile)) throw new Error('Slack users.info omitted the user profile');

	return {
		...profile,
		slack_display_name: optionalString(slackProfile.display_name)?.trim() || slackId,
		slack_image_512: optionalString(slackProfile.image_512) ?? null
	};
}

export function mapHackClubProfile(profile: Record<string, unknown>, checkedAt = new Date()) {
	return {
		id: optionalString(profile.sub),
		name: optionalString(profile.slack_display_name) ?? optionalString(profile.slack_id),
		email: optionalString(profile.email),
		emailVerified: profile.email_verified === true,
		image: optionalString(profile.slack_image_512) ?? null,
		slackId: optionalString(profile.slack_id),
		verificationStatus: optionalString(profile.verification_status),
		yswsEligible: profile.ysws_eligible === true,
		pronouns: optionalString(profile.pronouns),
		profileCheckedAt: checkedAt
	};
}

function normalizeProfile(profile: Record<string, unknown>): HackClubProfile | null {
	const id = optionalString(profile.sub) ?? optionalString(profile.id);
	const email = optionalString(profile.email);
	if (!id || !email) return null;

	return {
		id,
		sub: id,
		email,
		emailVerified: profile.email_verified === true,
		email_verified: profile.email_verified === true,
		slack_id: optionalString(profile.slack_id),
		verification_status: optionalString(profile.verification_status),
		ysws_eligible: profile.ysws_eligible === true,
		pronouns: optionalString(profile.pronouns)
	};
}

function optionalString(value: unknown): string | undefined {
	return typeof value === 'string' ? value : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}
