import { describe, expect, it } from 'vitest';
import { userProfileInputFromForm, UserProfileValidationError } from './user-profile';

function profileForm(values: Record<string, string> = {}) {
	const formData = new FormData();
	for (const [key, value] of Object.entries(values)) formData.set(key, value);
	return formData;
}

describe('user submission profile', () => {
	it('accepts and normalizes a valid birthday', () => {
		const profile = userProfileInputFromForm(profileForm({ birthday: '2000-02-29' }), {
			requireBirthday: true
		});

		expect(profile.birthday).toBe('2000-02-29');
	});

	it('requires a birthday for project submissions', () => {
		expect(() => userProfileInputFromForm(profileForm(), { requireBirthday: true })).toThrowError(
			new UserProfileValidationError('Birthday is required.')
		);
	});

	it.each(['2001-02-29', 'not-a-date'])('rejects invalid birthday %s', (birthday) => {
		expect(() => userProfileInputFromForm(profileForm({ birthday }))).toThrowError(
			new UserProfileValidationError('Enter a valid birthday.')
		);
	});

	it('rejects a future birthday', () => {
		expect(() => userProfileInputFromForm(profileForm({ birthday: '2999-01-01' }))).toThrowError(
			new UserProfileValidationError('Birthday cannot be in the future.')
		);
	});

	it('allows birthday to be cleared in settings', () => {
		expect(userProfileInputFromForm(profileForm({ birthday: ' ' })).birthday).toBeNull();
	});
});
