import { and, asc, desc, eq, gte, ne, sql } from 'drizzle-orm';
import { alias } from 'drizzle-orm/pg-core';
import { db } from '$lib/server/db';
import { account as authAccount } from '$lib/server/db/auth.schema';
import { project, shopItem, shopOrder, user } from '$lib/server/db/schema';
import {
	getShopEligibility,
	HACKXPANSION_CONSOLE,
	isShopItemUnlocked,
	type ShopProgress
} from '$lib/shop/domain';
import type { CatalogItemInput } from '$lib/shop/catalog';

const MAX_NOTE_LENGTH = 2_000;
const CONFIGURED_ADMIN_HACKCLUB_ID = 'ident!ZVpfLg';
const fulfiller = alias(user, 'fulfiller');

export class ShopError extends Error {
	constructor(
		readonly status: number,
		message: string
	) {
		super(message);
		this.name = 'ShopError';
	}
}

export async function getShopCatalog(userId?: string) {
	const [databaseItems, progress, balance, hasConsoleOrder] = await Promise.all([
		db
			.select()
			.from(shopItem)
			.where(and(eq(shopItem.active, true), ne(shopItem.id, HACKXPANSION_CONSOLE.id)))
			.orderBy(asc(shopItem.sortOrder), asc(shopItem.name)),
		userId ? getShopProgress(userId) : Promise.resolve({ moduleDesigns: 0, appDesigns: 0 }),
		userId
			? db
					.select({ currency: user.currency })
					.from(user)
					.where(eq(user.id, userId))
					.limit(1)
					.then((rows) => rows[0]?.currency ?? 0)
			: Promise.resolve(0),
		userId ? hasOrderedConsole(userId) : Promise.resolve(false)
	]);
	const items = [HACKXPANSION_CONSOLE, ...databaseItems];

	return {
		balance,
		progress,
		shopUnlocked: hasConsoleOrder,
		items: items.map((item) => ({
			...item,
			requiredModuleDesigns:
				item.id === HACKXPANSION_CONSOLE.id ? HACKXPANSION_CONSOLE.requiredModuleDesigns : 0,
			requiredAppDesigns:
				item.id === HACKXPANSION_CONSOLE.id ? HACKXPANSION_CONSOLE.requiredAppDesigns : 0,
			eligibility:
				item.id === HACKXPANSION_CONSOLE.id
					? getShopEligibility(HACKXPANSION_CONSOLE, progress)
					: getShopEligibility({ requiredModuleDesigns: 0, requiredAppDesigns: 0 }, progress),
			unlocked: isShopItemUnlocked(item.id, hasConsoleOrder),
			canOrder:
				Boolean(userId) &&
				item.price <= balance &&
				isShopItemUnlocked(item.id, hasConsoleOrder) &&
				(item.id !== HACKXPANSION_CONSOLE.id ||
					getShopEligibility(HACKXPANSION_CONSOLE, progress).eligible)
		}))
	};
}

export async function createShopOrder(userId: string, itemId: string, rawNotes: string) {
	const notes = optionalNote(rawNotes, 'Notes');

	return db.transaction(async (tx) => {
		const isConsole = itemId === HACKXPANSION_CONSOLE.id;
		if (isConsole) {
			await tx
				.insert(shopItem)
				.values({
					id: HACKXPANSION_CONSOLE.id,
					name: HACKXPANSION_CONSOLE.name,
					description: HACKXPANSION_CONSOLE.description,
					price: HACKXPANSION_CONSOLE.price,
					imageUrl: HACKXPANSION_CONSOLE.imageUrl,
					active: HACKXPANSION_CONSOLE.active,
					sortOrder: HACKXPANSION_CONSOLE.sortOrder
				})
				.onConflictDoUpdate({
					target: shopItem.id,
					set: {
						name: HACKXPANSION_CONSOLE.name,
						description: HACKXPANSION_CONSOLE.description,
						price: HACKXPANSION_CONSOLE.price,
						imageUrl: HACKXPANSION_CONSOLE.imageUrl,
						active: HACKXPANSION_CONSOLE.active,
						sortOrder: HACKXPANSION_CONSOLE.sortOrder
					}
				});
		}
		const [databaseItem] = await tx
			.select()
			.from(shopItem)
			.where(
				isConsole
					? eq(shopItem.id, HACKXPANSION_CONSOLE.id)
					: and(eq(shopItem.id, itemId), eq(shopItem.active, true))
			)
			.limit(1)
			.for('update');
		if (!databaseItem) throw new ShopError(404, 'Shop item not found');
		const item = isConsole ? HACKXPANSION_CONSOLE : databaseItem;

		if (isConsole) {
			const acceptedDesigns = await tx
				.select({ type: project.designApprovedType })
				.from(project)
				.where(and(eq(project.userId, userId), eq(project.designCurrencyAwarded, true)))
				.orderBy(asc(project.id))
				.for('update', { of: project });
			const eligibility = getShopEligibility(
				HACKXPANSION_CONSOLE,
				countAcceptedDesigns(acceptedDesigns)
			);
			if (!eligibility.eligible) {
				throw new ShopError(422, eligibilityMessage(eligibility));
			}
		} else if (!(await hasOrderedConsole(userId, tx))) {
			throw new ShopError(422, 'Buy the Hackxpansion Console before ordering other shop items.');
		}

		const [chargedUser] = await tx
			.update(user)
			.set({ currency: sql`${user.currency} - ${item.price}` })
			.where(and(eq(user.id, userId), gte(user.currency, item.price)))
			.returning({ currency: user.currency });
		if (!chargedUser)
			throw new ShopError(422, `You need ${item.price} currency to order this item.`);

		const [order] = await tx
			.insert(shopOrder)
			.values({ itemId: item.id, userId, pricePaid: item.price, notes })
			.returning({ id: shopOrder.id });

		return { orderId: order.id, itemName: item.name, balance: chargedUser.currency };
	});
}

export async function getUserShopOrders(userId: string) {
	const orders = await db
		.select({
			id: shopOrder.id,
			itemId: shopOrder.itemId,
			status: shopOrder.status,
			pricePaid: shopOrder.pricePaid,
			notes: shopOrder.notes,
			fulfillmentMessage: shopOrder.fulfillmentMessage,
			createdAt: shopOrder.createdAt,
			fulfilledAt: shopOrder.fulfilledAt,
			fulfilledByUserId: shopOrder.fulfilledByUserId,
			fulfillerName: fulfiller.name,
			itemName: shopItem.name
		})
		.from(shopOrder)
		.innerJoin(shopItem, eq(shopOrder.itemId, shopItem.id))
		.leftJoin(fulfiller, eq(shopOrder.fulfilledByUserId, fulfiller.id))
		.where(eq(shopOrder.userId, userId))
		.orderBy(desc(shopOrder.createdAt));
	return orders.map((order) => ({
		...order,
		itemName: order.itemId === HACKXPANSION_CONSOLE.id ? HACKXPANSION_CONSOLE.name : order.itemName
	}));
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
	if (!(await isUserAdmin(userId))) throw new ShopError(404, 'Page not found');
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
	if (!targetUser) throw new ShopError(404, 'User not found.');
	throw new ShopError(409, 'This user is already an admin.');
}

export async function demoteUserFromAdmin(adminUserId: string, targetUserId: string) {
	await requireAdmin(adminUserId);
	if (targetUserId === (await getConfiguredAdminUserId())) {
		throw new ShopError(422, 'The configured admin cannot be demoted.');
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
	if (!targetUser) throw new ShopError(404, 'User not found.');
	throw new ShopError(409, 'This user is not an admin.');
}

export async function getAdminShopItems() {
	return db
		.select()
		.from(shopItem)
		.where(ne(shopItem.id, HACKXPANSION_CONSOLE.id))
		.orderBy(asc(shopItem.sortOrder), asc(shopItem.name));
}

export async function createCatalogItem(adminUserId: string, input: CatalogItemInput) {
	await requireAdmin(adminUserId);
	if (input.id === HACKXPANSION_CONSOLE.id) {
		throw new ShopError(422, 'The Hackxpansion Console is managed in code.');
	}

	const created = await db
		.insert(shopItem)
		.values(input)
		.onConflictDoNothing({ target: shopItem.id })
		.returning({ id: shopItem.id });
	if (created.length === 0) throw new ShopError(409, 'An item with this ID already exists.');
}

export async function updateCatalogItem(
	adminUserId: string,
	itemId: string,
	input: CatalogItemInput
) {
	await requireAdmin(adminUserId);
	if (itemId === HACKXPANSION_CONSOLE.id || input.id !== itemId) {
		throw new ShopError(422, 'This shop item cannot be edited.');
	}

	const updated = await db
		.update(shopItem)
		.set({
			name: input.name,
			description: input.description,
			price: input.price,
			imageUrl: input.imageUrl,
			sortOrder: input.sortOrder,
			active: input.active
		})
		.where(and(eq(shopItem.id, itemId), ne(shopItem.id, HACKXPANSION_CONSOLE.id)))
		.returning({ id: shopItem.id });
	if (updated.length === 0) throw new ShopError(404, 'Shop item not found.');
}

export async function getAllShopOrders() {
	const orders = await db
		.select({
			id: shopOrder.id,
			itemId: shopOrder.itemId,
			status: shopOrder.status,
			pricePaid: shopOrder.pricePaid,
			notes: shopOrder.notes,
			fulfillmentMessage: shopOrder.fulfillmentMessage,
			createdAt: shopOrder.createdAt,
			fulfilledAt: shopOrder.fulfilledAt,
			fulfilledByUserId: shopOrder.fulfilledByUserId,
			fulfillerName: fulfiller.name,
			itemName: shopItem.name,
			userName: user.name,
			userEmail: user.email
		})
		.from(shopOrder)
		.innerJoin(shopItem, eq(shopOrder.itemId, shopItem.id))
		.innerJoin(user, eq(shopOrder.userId, user.id))
		.leftJoin(fulfiller, eq(shopOrder.fulfilledByUserId, fulfiller.id))
		.orderBy(
			sql`CASE WHEN ${shopOrder.status} = 'in_queue' THEN 0 ELSE 1 END`,
			asc(shopOrder.createdAt)
		);
	return orders.map((order) => ({
		...order,
		itemName: order.itemId === HACKXPANSION_CONSOLE.id ? HACKXPANSION_CONSOLE.name : order.itemName
	}));
}

export async function fulfillShopOrder(adminUserId: string, orderId: string, rawMessage: string) {
	const fulfillmentMessage = optionalNote(rawMessage, 'Fulfillment message');

	return db.transaction(async (tx) => {
		const [admin] = await tx
			.select({ isAdmin: user.isAdmin })
			.from(user)
			.where(eq(user.id, adminUserId))
			.limit(1);
		if (!admin?.isAdmin) throw new ShopError(404, 'Page not found');

		const [order] = await tx
			.select({ id: shopOrder.id, status: shopOrder.status })
			.from(shopOrder)
			.where(eq(shopOrder.id, orderId))
			.limit(1)
			.for('update');
		if (!order) throw new ShopError(404, 'Order not found');
		if (order.status === 'fulfilled') throw new ShopError(409, 'This order is already fulfilled');

		await tx
			.update(shopOrder)
			.set({
				status: 'fulfilled',
				fulfillmentMessage,
				fulfilledAt: new Date(),
				fulfilledByUserId: adminUserId
			})
			.where(eq(shopOrder.id, orderId));
	});
}

function countAcceptedDesigns(rows: Array<{ type: 'card' | 'app' | null }>): ShopProgress {
	return {
		moduleDesigns: rows.filter((row) => row.type === 'card').length,
		appDesigns: rows.filter((row) => row.type === 'app').length
	};
}

async function getShopProgress(userId: string) {
	const rows = await db
		.select({ type: project.designApprovedType })
		.from(project)
		.where(and(eq(project.userId, userId), eq(project.designCurrencyAwarded, true)));
	return countAcceptedDesigns(rows);
}

async function hasOrderedConsole(userId: string, database: Pick<typeof db, 'select'> = db) {
	const rows = await database
		.select({ id: shopOrder.id })
		.from(shopOrder)
		.where(and(eq(shopOrder.userId, userId), eq(shopOrder.itemId, HACKXPANSION_CONSOLE.id)))
		.limit(1);
	return rows.length > 0;
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

function optionalNote(value: string, label: string) {
	const trimmed = value.trim();
	if (trimmed.length > MAX_NOTE_LENGTH) {
		throw new ShopError(422, `${label} must be ${MAX_NOTE_LENGTH} characters or fewer.`);
	}
	return trimmed || null;
}

function eligibilityMessage(eligibility: {
	missingModuleDesigns: number;
	missingAppDesigns: number;
}) {
	const requirements = [];
	if (eligibility.missingModuleDesigns > 0) {
		requirements.push(
			`${eligibility.missingModuleDesigns} more module design${eligibility.missingModuleDesigns === 1 ? '' : 's'}`
		);
	}
	if (eligibility.missingAppDesigns > 0) {
		requirements.push(
			`${eligibility.missingAppDesigns} more app design${eligibility.missingAppDesigns === 1 ? '' : 's'}`
		);
	}
	return `You need ${requirements.join(' and ')} accepted before ordering this item.`;
}
