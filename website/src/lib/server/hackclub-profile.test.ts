import { describe, expect, it } from 'vitest';
import { mapHackClubProfile } from './hackclub-profile';

describe('Hack Club profile mapping', () => {
	it('maps provider fields to the local user schema', () => {
		const checkedAt = new Date('2026-08-03T12:00:00Z');

		expect(
			mapHackClubProfile(
				{
					sub: 'ident!example',
					name: 'Example User',
					email: 'USER@example.com',
					email_verified: true,
					imageUrl: 'https://example.com/avatar.png',
					slack_id: 'U123',
					verification_status: 'verified',
					given_name: 'Example',
					ysws_eligible: true,
					pronouns: 'they/them'
				},
				checkedAt
			)
		).toEqual({
			id: 'ident!example',
			name: 'Example User',
			email: 'USER@example.com',
			emailVerified: true,
			image: 'https://example.com/avatar.png',
			slackId: 'U123',
			verificationStatus: 'verified',
			given_name: 'Example',
			yswsEligible: true,
			pronouns: 'they/them',
			profileCheckedAt: checkedAt
		});
	});

	it('does not coerce malformed optional values', () => {
		const mapped = mapHackClubProfile({ ysws_eligible: 'true', imageUrl: 123 });

		expect(mapped.yswsEligible).toBe(false);
		expect(mapped.image).toBeUndefined();
	});
});
