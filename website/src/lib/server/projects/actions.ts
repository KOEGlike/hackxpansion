import { fail } from '@sveltejs/kit';
import { AriInboundError } from '$lib/server/ari/inbound';
import { HackatimeError } from '$lib/server/hackatime';
import {
	formValuesToObject,
	getInvalidProjectHackatimeProjects,
	projectInputFromForm
} from '$lib/server/projects/form';
import { ProjectMutationError, type ProjectInput } from '$lib/server/projects/mutations';
import {
	ProjectSubmissionError,
	submitProjectToAri,
	withdrawProjectFromAri
} from '$lib/server/projects/submit';
import {
	projectSubmissionFeedbackFormValues,
	projectSubmissionFeedbackFromForm,
	ProjectSubmissionFeedbackError
} from '$lib/server/projects/feedback';
import {
	userProfileFormValues,
	userProfileInputFromForm,
	UserProfileValidationError
} from '$lib/server/user-profile';
import { internalErrorDetails, upstreamResponseExcerpt } from '$lib/server/error-logging';
import { currentRequestId } from '$lib/server/request-context';
import {
	journalInputFromForm,
	JournalMutationError,
	type JournalInput
} from '$lib/server/projects/journals';

export async function handleProjectFormAction({
	request,
	slackId,
	mutate
}: {
	request: Request;
	slackId: string;
	mutate: (input: ProjectInput) => Promise<unknown>;
}) {
	const formData = await request.formData();
	const values = formValuesToObject(formData);

	try {
		const input = projectInputFromForm(formData);
		const invalidProjects = await getInvalidProjectHackatimeProjects(
			slackId,
			input.hackatimeProjects ?? []
		);

		if (invalidProjects.length > 0) {
			return fail(422, {
				success: false,
				message: `Invalid Hackatime projects: ${invalidProjects.join(', ')}`,
				values
			});
		}

		await mutate(input);
		return null;
	} catch (error) {
		if (error instanceof ProjectMutationError) {
			return fail(error.status, { success: false, message: error.message, values });
		}
		if (error instanceof HackatimeError) {
			return fail(503, {
				success: false,
				message: 'Could not validate Hackatime projects. Try again shortly.',
				values
			});
		}

		console.error('[projects] Unexpected project form error', error);
		return fail(500, { success: false, message: 'Something went wrong.', values });
	}
}

export async function submitProjectAction(projectId: string, userId: string, formData: FormData) {
	const values = {
		...userProfileFormValues(formData),
		...projectSubmissionFeedbackFormValues(formData)
	};

	try {
		const profile = userProfileInputFromForm(formData, {
			requireAddress: true,
			requireBirthday: true
		});
		const feedback = projectSubmissionFeedbackFromForm(formData);
		const result = await submitProjectToAri({ projectId, userId, profile, feedback });
		return {
			success: true,
			message: `Submitted for ${result.phase} review.`,
			projectId
		};
	} catch (error) {
		if (
			error instanceof UserProfileValidationError ||
			error instanceof ProjectSubmissionFeedbackError
		) {
			return fail(422, { success: false, message: error.message, projectId, values });
		}
		return projectReviewActionFailure(error, projectId, values);
	}
}

export async function withdrawProjectAction(projectId: string, userId: string) {
	try {
		await withdrawProjectFromAri({ projectId, userId });
		return { success: true, message: 'Project withdrawn from review.', projectId };
	} catch (error) {
		return projectReviewActionFailure(error, projectId);
	}
}

export async function handleJournalAction(
	request: Request,
	mutate: (input: JournalInput, formData: FormData) => Promise<unknown>
) {
	const formData = await request.formData();
	try {
		await mutate(journalInputFromForm(formData), formData);
		return { journalSuccess: true };
	} catch (error) {
		if (error instanceof JournalMutationError) {
			return fail(error.status, { journalError: error.message });
		}

		console.error('[projects] Unexpected journal action error', error);
		return fail(500, { journalError: 'Something went wrong.' });
	}
}

function projectReviewActionFailure(
	error: unknown,
	projectId: string,
	values?: Record<string, string>
) {
	if (error instanceof ProjectSubmissionError) {
		if (error.status >= 500) {
			console.error('[projects/review] Internal submission error', {
				requestId: currentRequestId(),
				status: error.status,
				error: internalErrorDetails(error.cause ?? error)
			});
			const ariError = findAriInboundError(error.cause);
			if (ariError) logAriRejection(ariError);
		}
		return fail(error.status, {
			success: false,
			message: error.message,
			projectId,
			values
		});
	}
	if (error instanceof AriInboundError) {
		logAriRejection(error);
		return fail(error.status, {
			success: false,
			message: 'The review service could not process this request. Please try again.',
			projectId,
			values
		});
	}

	console.error('[projects] Unexpected Ari action error', error);
	return fail(500, { success: false, message: 'Something went wrong.', projectId, values });
}

function logAriRejection(error: AriInboundError) {
	console.error('[ari/review] Ari rejected the request', {
		requestId: currentRequestId(),
		status: error.status,
		response: upstreamResponseExcerpt(error.responseBody)
	});
}

function findAriInboundError(error: unknown): AriInboundError | null {
	if (error instanceof AriInboundError) return error;
	if (error instanceof AggregateError) {
		for (const item of error.errors) {
			const found = findAriInboundError(item);
			if (found) return found;
		}
	}
	if (error instanceof Error && error.cause) return findAriInboundError(error.cause);
	return null;
}
