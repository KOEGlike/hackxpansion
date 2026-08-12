import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	requireAdmin: vi.fn(),
	deleteReturning: vi.fn(),
	selectLimit: vi.fn(),
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

const projectId = '019ff242-5431-7ad2-afef-ac36673bd5ac';
const deleteAction = actions.delete!;

function actionEvent(id = projectId, user: { id: string } | null = { id: 'admin-1' }) {
	return { locals: { user }, params: { id } } as unknown as Parameters<typeof deleteAction>[0];
}

beforeEach(() => {
	vi.clearAllMocks();
	mocks.requireAdmin.mockResolvedValue(undefined);
	mocks.delete.mockReturnValue({
		where: vi.fn().mockReturnValue({ returning: mocks.deleteReturning })
	});
	mocks.select.mockReturnValue({
		from: vi.fn().mockReturnValue({
			where: vi.fn().mockReturnValue({ limit: mocks.selectLimit })
		})
	});
});

describe('admin project deletion', () => {
	it('rejects invalid project IDs before querying the database', async () => {
		await expect(deleteAction(actionEvent('not-a-uuid'))).resolves.toMatchObject({
			status: 400,
			data: { success: false, message: 'A valid project ID is required.' }
		});
		expect(mocks.delete).not.toHaveBeenCalled();
	});

	it('conceals the action from signed-out and non-admin users', async () => {
		await expect(deleteAction(actionEvent(projectId, null))).resolves.toMatchObject({
			status: 404
		});

		mocks.requireAdmin.mockRejectedValue(new AdminError(404, 'Page not found'));
		await expect(deleteAction(actionEvent())).resolves.toMatchObject({ status: 404 });
		expect(mocks.delete).not.toHaveBeenCalled();
	});

	it.each(['approved_design', 'rejected_build'])(
		'deletes a project in %s even when it retains an ARI ID',
		async () => {
			mocks.deleteReturning.mockResolvedValue([{ id: projectId }]);

			await expect(deleteAction(actionEvent())).rejects.toMatchObject({
				status: 303,
				location: '/home/admin/projects'
			});
			expect(mocks.select).not.toHaveBeenCalled();
		}
	);

	it.each(['waiting_design', 'waiting_build'])('blocks a project in %s', async (status) => {
		mocks.deleteReturning.mockResolvedValue([]);
		mocks.selectLimit.mockResolvedValue([{ status }]);

		await expect(deleteAction(actionEvent())).resolves.toMatchObject({
			status: 409,
			data: {
				success: false,
				message: 'Withdraw this project from ARI before deleting it.'
			}
		});
	});

	it('reports a project that no longer exists', async () => {
		mocks.deleteReturning.mockResolvedValue([]);
		mocks.selectLimit.mockResolvedValue([]);

		await expect(deleteAction(actionEvent())).resolves.toMatchObject({
			status: 404,
			data: { success: false, message: 'Project not found.' }
		});
	});
});
