import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	requireAdmin: vi.fn(),
	deleteReturning: vi.fn(),
	existingLimit: vi.fn(),
	delete: vi.fn(),
	select: vi.fn()
}));

vi.mock('$lib/server/db', () => ({
	db: {
		delete: mocks.delete,
		select: mocks.select
	}
}));

vi.mock('$lib/server/admin', () => {
	class AdminError extends Error {
		constructor(
			readonly status: number,
			message: string
		) {
			super(message);
		}
	}

	return { AdminError, requireAdmin: mocks.requireAdmin };
});

import { AdminError } from '$lib/server/admin';
import { actions } from './+page.server';

const submissionId = '019ff59c-69b3-785a-9618-a2ee6ae323a1';
const deleteAction = actions.delete!;

function actionEvent(id = submissionId, user: { id: string } | null = { id: 'admin-1' }) {
	const formData = new FormData();
	formData.set('submissionId', id);
	return {
		locals: { user },
		request: new Request('https://example.com/home/admin/submissions?/delete', {
			method: 'POST',
			body: formData
		})
	} as unknown as Parameters<typeof deleteAction>[0];
}

beforeEach(() => {
	vi.clearAllMocks();
	mocks.requireAdmin.mockResolvedValue(undefined);
	mocks.select.mockReturnValue({
		from: vi.fn().mockReturnValue({
			where: vi.fn().mockReturnValue({
				innerJoin: vi.fn(),
				limit: mocks.existingLimit
			})
		})
	});
	mocks.delete.mockReturnValue({
		where: vi.fn().mockReturnValue({ returning: mocks.deleteReturning })
	});
});

describe('admin submission deletion', () => {
	it('rejects invalid submission IDs before querying the database', async () => {
		await expect(deleteAction(actionEvent('not-a-uuid'))).resolves.toMatchObject({
			status: 400,
			data: { success: false, message: 'A valid submission ID is required.' }
		});
		expect(mocks.delete).not.toHaveBeenCalled();
	});

	it('conceals the action from signed-out and non-admin users', async () => {
		await expect(deleteAction(actionEvent(submissionId, null))).resolves.toMatchObject({
			status: 404
		});

		mocks.requireAdmin.mockRejectedValue(new AdminError(404, 'Page not found'));
		await expect(deleteAction(actionEvent())).resolves.toMatchObject({ status: 404 });
		expect(mocks.delete).not.toHaveBeenCalled();
	});

	it('deletes a submission that is not waiting for review', async () => {
		mocks.deleteReturning.mockResolvedValue([{ id: submissionId }]);

		await expect(deleteAction(actionEvent())).resolves.toEqual({
			success: true,
			message: 'Submission deleted.',
			submissionId
		});
	});

	it('blocks an active submission that is waiting for ARI review', async () => {
		mocks.deleteReturning.mockResolvedValue([]);
		mocks.select
			.mockReturnValueOnce({
				from: vi.fn().mockReturnValue({ where: vi.fn().mockReturnValue({}) })
			})
			.mockReturnValueOnce({
				from: vi.fn().mockReturnValue({
					innerJoin: vi.fn().mockReturnValue({
						where: vi.fn().mockReturnValue({ limit: mocks.existingLimit })
					})
				})
			});
		mocks.existingLimit.mockResolvedValue([
			{
				projectStatus: 'waiting_design',
				activeAriExternalId: 'ari-123',
				ariExternalId: 'ari-123'
			}
		]);

		await expect(deleteAction(actionEvent())).resolves.toMatchObject({
			status: 409,
			data: {
				success: false,
				message: 'This submission cannot be deleted while it is waiting for ARI review.',
				submissionId
			}
		});
	});

	it('reports a submission that no longer exists', async () => {
		mocks.deleteReturning.mockResolvedValue([]);
		mocks.select
			.mockReturnValueOnce({
				from: vi.fn().mockReturnValue({ where: vi.fn().mockReturnValue({}) })
			})
			.mockReturnValueOnce({
				from: vi.fn().mockReturnValue({
					innerJoin: vi.fn().mockReturnValue({
						where: vi.fn().mockReturnValue({ limit: mocks.existingLimit })
					})
				})
			});
		mocks.existingLimit.mockResolvedValue([]);

		await expect(deleteAction(actionEvent())).resolves.toMatchObject({
			status: 404,
			data: { success: false, message: 'Submission not found.', submissionId }
		});
	});
});
