import { fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { requireUser } from '$lib/server/guards';
import {
	getUserSubmissionProfile,
	updateUserSubmissionProfile,
	userProfileFormValues,
	userProfileInputFromForm,
	UserProfileValidationError
} from '$lib/server/user-profile';
import { refreshHackClubProfile } from '$lib/server/profile-sync';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) return {} as never;
	const user = requireUser(locals);
	return { profile: await getUserSubmissionProfile(user.id) };
};

export const actions: Actions = {
	updateProfile: async ({ locals, request }) => {
		const user = requireUser(locals);
		const formData = await request.formData();
		const profileValues = userProfileFormValues(formData);

		try {
			await updateUserSubmissionProfile(user.id, userProfileInputFromForm(formData));
			return { profileSuccess: true, message: 'Submission details saved.' };
		} catch (error) {
			if (error instanceof UserProfileValidationError) {
				return fail(422, { profileSuccess: false, message: error.message, profileValues });
			}

			console.error('[settings] Unexpected profile update error', error);
			return fail(500, {
				profileSuccess: false,
				message: 'Could not save submission details.',
				profileValues
			});
		}
	},
	refreshHackClubProfile: async ({ locals, request }) => {
		const user = requireUser(locals);

		try {
			await refreshHackClubProfile(user.id, request.headers);
			return {
				hackClubProfileSuccess: true,
				message: 'Hack Club profile refreshed.'
			};
		} catch (error) {
			console.error('[settings] Could not refresh Hack Club profile', error);
			return fail(502, {
				hackClubProfileSuccess: false,
				message: 'Could not refresh your Hack Club profile. Try signing in again.'
			});
		}
	}
};
