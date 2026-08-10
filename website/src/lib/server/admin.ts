import { and, asc, desc, eq } from 'drizzle-orm';
import { db } from '$lib/server/db';
import { account as authAccount } from '$lib/server/db/auth.schema';
import { user } from '$lib/server/db/schema';

const CONFIGURED_ADMIN_HACKCLUB_ID = 'ident!ZVpfLg';

export class AdminError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'AdminError';
	}
}

export async function isUserAdmin(userId: string) {
	await ensureConfiguredAdmin();

	const [row] = await db
		.select({ isAdmin: user.isAdmin })
		.from(user)
		.where(eq(user.id, userId))
		.limit(1);
	return row?.isAdmin === true;
}

export async function requireAdmin(userId: string) {
	if (!(await isUserAdmin(userId))) throw new AdminError(404, 'Page not found');
}

export async function getAdminUsers() {
	const users = await db
		.select({
			id: user.id,
			name: user.name,
			email: user.email,
			slackId: user.slackId,
			isAdmin: user.isAdmin,
			createdAt: user.createdAt
		})
		.from(user)
		.orderBy(desc(user.isAdmin), asc(user.name), asc(user.email));
	const configuredAdminUserId = await getConfiguredAdminUserId();
	return users.map((account) => ({
		...account,
		isProtectedAdmin: account.id === configuredAdminUserId
	}));
}

export async function promoteUserToAdmin(adminUserId: string, targetUserId: string) {
	await requireAdmin(adminUserId);
	const [promotedUser] = await db
		.update(user)
		.set({ isAdmin: true })
		.where(and(eq(user.id, targetUserId), eq(user.isAdmin, false)))
		.returning({ name: user.name });
	if (promotedUser) return promotedUser;

	const [targetUser] = await db
		.select({ isAdmin: user.isAdmin })
		.from(user)
		.where(eq(user.id, targetUserId))
		.limit(1);
	if (!targetUser) throw new AdminError(404, 'User not found.');
	throw new AdminError(409, 'This user is already an admin.');
}

export async function demoteUserFromAdmin(adminUserId: string, targetUserId: string) {
	await requireAdmin(adminUserId);
	if (targetUserId === (await getConfiguredAdminUserId())) {
		throw new AdminError(422, 'The configured admin cannot be demoted.');
	}

	const [demotedUser] = await db
		.update(user)
		.set({ isAdmin: false })
		.where(and(eq(user.id, targetUserId), eq(user.isAdmin, true)))
		.returning({ name: user.name });
	if (demotedUser) return demotedUser;

	const [targetUser] = await db
		.select({ isAdmin: user.isAdmin })
		.from(user)
		.where(eq(user.id, targetUserId))
		.limit(1);
	if (!targetUser) throw new AdminError(404, 'User not found.');
	throw new AdminError(409, 'This user is not an admin.');
}

async function getConfiguredAdminUserId() {
	const [configuredAccount] = await db
		.select({ userId: authAccount.userId })
		.from(authAccount)
		.where(
			and(
				eq(authAccount.providerId, 'hackclub'),
				eq(authAccount.accountId, CONFIGURED_ADMIN_HACKCLUB_ID)
			)
		)
		.limit(1);
	return configuredAccount?.userId ?? null;
}

async function ensureConfiguredAdmin() {
	const configuredAdminUserId = await getConfiguredAdminUserId();
	if (!configuredAdminUserId) return;
	await db
		.update(user)
		.set({ isAdmin: true })
		.where(and(eq(user.id, configuredAdminUserId), eq(user.isAdmin, false)));
}
