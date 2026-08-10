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
	}
};
