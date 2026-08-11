import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	fetchHackClubProfile,
	fetchHackClubRealName,
	mapHackClubProfile
} from './hackclub-profile';

afterEach(() => vi.unstubAllGlobals());

describe('Hack Club profile mapping', () => {
	it('maps provider fields to the local user schema', () => {
		const checkedAt = new Date('2026-08-03T12:00:00Z');

		expect(
			mapHackClubProfile(
				{
					sub: 'ident!example',
					name: 'Private Real Name',
					email: 'USER@example.com',
					email_verified: true,
					slack_display_name: 'Example Display',
					slack_image_512: 'https://example.com/slack-avatar.png',
					slack_id: 'U123',
					verification_status: 'verified',
					ysws_eligible: true,
					pronouns: 'they/them'
				},
				checkedAt
			)
		).toEqual({
			id: 'ident!example',
			name: 'Example Display',
			email: 'USER@example.com',
			emailVerified: true,
			image: 'https://example.com/slack-avatar.png',
			slackId: 'U123',
			verificationStatus: 'verified',
			yswsEligible: true,
			pronouns: 'they/them',
			profileCheckedAt: checkedAt
		});
	});

	it('loads display name and avatar directly from Slack', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(
				new Response(
					JSON.stringify({
						sub: 'ident!example',
						name: 'Private Real Name',
						email: 'user@example.com',
						slack_id: 'U123'
					}),
					{ status: 200 }
				)
			)
			.mockResolvedValueOnce(
				new Response(
					JSON.stringify({
						ok: true,
						user: {
							profile: {
								display_name: 'Slack Display',
								image_512: 'https://example.com/slack.png'
							}
						}
					}),
					{ status: 200 }
				)
			);
		vi.stubGlobal('fetch', fetchMock);

		const profile = await fetchHackClubProfile('oauth-token', 'slack-token');
		expect(mapHackClubProfile(profile ?? {})).toMatchObject({
			name: 'Slack Display',
			image: 'https://example.com/slack.png'
		});
		const [slackUrl, slackInit] = fetchMock.mock.calls[1] as [URL, RequestInit];
		expect(slackUrl.toString()).toBe('https://slack.com/api/users.info?user=U123');
		expect(slackInit.method).toBe('POST');
		expect(new Headers(slackInit.headers).get('Authorization')).toBe('Bearer slack-token');
		expect(slackInit.body).toBe(JSON.stringify({ user: 'U123' }));
	});

	it('does not coerce malformed optional values', () => {
		const mapped = mapHackClubProfile({ ysws_eligible: 'true', imageUrl: 123 });

		expect(mapped.yswsEligible).toBe(false);
		expect(mapped.image).toBeNull();
	});

	it('reads the real name only through the dedicated Airtable helper', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(
					JSON.stringify({
						sub: 'ident!example',
						name: 'Private Real Name',
						email: 'user@example.com'
					}),
					{ status: 200 }
				)
			)
		);

		await expect(fetchHackClubRealName('oauth-token')).resolves.toBe('Private Real Name');
	});

	it('does not silently accept a failed Slack lookup', async () => {
		vi.stubGlobal(
			'fetch',
			vi
				.fn()
				.mockResolvedValueOnce(
					new Response(
						JSON.stringify({
							sub: 'ident!example',
							email: 'user@example.com',
							slack_id: 'U123'
						}),
						{ status: 200 }
					)
				)
				.mockResolvedValueOnce(
					new Response(JSON.stringify({ ok: false, error: 'invalid_auth' }), { status: 200 })
				)
		);

		await expect(fetchHackClubProfile('oauth-token', 'bad-token')).rejects.toThrow(
			'Slack users.info returned an invalid response'
		);
	});
});
