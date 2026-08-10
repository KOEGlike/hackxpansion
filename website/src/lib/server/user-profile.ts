import { db } from '$lib/server/db';
import { user } from '$lib/server/db/auth.schema';
import { inferGithubUsername, type UserSubmissionProfile } from '$lib/profile';
import { eq } from 'drizzle-orm';

export type UserProfileFormValues = Record<keyof UserSubmissionProfile, string>;

export class UserProfileValidationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = 'UserProfileValidationError';
	}
}

export async function getUserSubmissionProfile(userId: string): Promise<UserSubmissionProfile> {
	const [profile] = await db
		.select({
			githubUsername: user.githubUsername,
			addressLine1: user.addressLine1,
			addressLine2: user.addressLine2,
			addressCity: user.addressCity,
			addressRegion: user.addressRegion,
			addressPostalCode: user.addressPostalCode,
			addressCountry: user.addressCountry
		})
		.from(user)
		.where(eq(user.id, userId))
		.limit(1);

	if (!profile) throw new UserProfileValidationError('User profile not found.');
	return profile;
}

export function userProfileFormValues(formData: FormData): UserProfileFormValues {
	return {
		githubUsername: stringFromForm(formData, 'githubUsername'),
		addressLine1: stringFromForm(formData, 'addressLine1'),
		addressLine2: stringFromForm(formData, 'addressLine2'),
		addressCity: stringFromForm(formData, 'addressCity'),
		addressRegion: stringFromForm(formData, 'addressRegion'),
		addressPostalCode: stringFromForm(formData, 'addressPostalCode'),
		addressCountry: stringFromForm(formData, 'addressCountry')
	};
}

export function userProfileInputFromForm(
	formData: FormData,
	options: { requireAddress?: boolean; repoUrl?: string | null } = {}
): UserSubmissionProfile {
	const values = userProfileFormValues(formData);
	const profile = {
		githubUsername: optionalValue(values.githubUsername, 39, 'GitHub username'),
		addressLine1: optionalValue(values.addressLine1, 200, 'Address line 1'),
		addressLine2: optionalValue(values.addressLine2, 200, 'Address line 2'),
		addressCity: optionalValue(values.addressCity, 100, 'City'),
		addressRegion: optionalValue(values.addressRegion, 100, 'State or province'),
		addressPostalCode: optionalValue(values.addressPostalCode, 32, 'ZIP or postal code'),
		addressCountry: optionalValue(values.addressCountry, 100, 'Country')
	} satisfies UserSubmissionProfile;

	profile.githubUsername ??= inferGithubUsername(options.repoUrl);
	if (profile.githubUsername && !/^(?!-)[A-Za-z0-9-]{1,39}(?<!-)$/.test(profile.githubUsername)) {
		throw new UserProfileValidationError('Enter a valid GitHub username.');
	}

	if (
		options.requireAddress &&
		(!profile.addressLine1 ||
			!profile.addressCity ||
			!profile.addressPostalCode ||
			!profile.addressCountry)
	) {
		throw new UserProfileValidationError(
			'Address line 1, city, ZIP or postal code, and country are required.'
		);
	}

	return profile;
}

export async function updateUserSubmissionProfile(userId: string, profile: UserSubmissionProfile) {
	await db.update(user).set(profile).where(eq(user.id, userId));
}

function optionalValue(value: string, maxLength: number, label: string) {
	const normalized = value.trim();
	if (normalized.length > maxLength) {
		throw new UserProfileValidationError(`${label} must be ${maxLength} characters or fewer.`);
	}
	return normalized || null;
}

function stringFromForm(formData: FormData, key: string) {
	const value = formData.get(key);
	return typeof value === 'string' ? value : '';
}
