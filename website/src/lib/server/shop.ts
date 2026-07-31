import { and, asc, desc, eq, gte, sql } from 'drizzle-orm';
import { db } from '$lib/server/db';
import { project, shopItem, shopOrder, user } from '$lib/server/db/schema';
import { getShopEligibility, type ShopProgress } from '$lib/shop/domain';

const MAX_NOTE_LENGTH = 2_000;

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
	const [items, progress, balance] = await Promise.all([
		db
			.select()
			.from(shopItem)
			.where(eq(shopItem.active, true))
			.orderBy(asc(shopItem.sortOrder), asc(shopItem.name)),
		userId ? getShopProgress(userId) : Promise.resolve({ moduleDesigns: 0, appDesigns: 0 }),
		userId
			? db
					.select({ currency: user.currency })
					.from(user)
					.where(eq(user.id, userId))
					.limit(1)
					.then((rows) => rows[0]?.currency ?? 0)
			: Promise.resolve(0)
	]);

	return {
		balance,
		progress,
		items: items.map((item) => ({
			...item,
			eligibility: getShopEligibility(item, progress),
			canOrder:
				Boolean(userId) && item.price <= balance && getShopEligibility(item, progress).eligible
		}))
	};
}

export async function createShopOrder(userId: string, itemId: string, rawNotes: string) {
	const notes = optionalNote(rawNotes, 'Notes');

	return db.transaction(async (tx) => {
		const [item] = await tx
			.select()
			.from(shopItem)
			.where(and(eq(shopItem.id, itemId), eq(shopItem.active, true)))
			.limit(1)
			.for('update');
		if (!item) throw new ShopError(404, 'Shop item not found');

		const acceptedDesigns = await tx
			.select({ type: project.designApprovedType })
			.from(project)
			.where(and(eq(project.userId, userId), eq(project.designCurrencyAwarded, true)))
			.orderBy(asc(project.id))
			.for('update', { of: project });
		const progress = countAcceptedDesigns(acceptedDesigns);
		const eligibility = getShopEligibility(item, progress);
		if (!eligibility.eligible) {
			throw new ShopError(422, eligibilityMessage(eligibility));
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
	return db
		.select({
			id: shopOrder.id,
			status: shopOrder.status,
			pricePaid: shopOrder.pricePaid,
			notes: shopOrder.notes,
			fulfillmentMessage: shopOrder.fulfillmentMessage,
			createdAt: shopOrder.createdAt,
			fulfilledAt: shopOrder.fulfilledAt,
			itemName: shopItem.name
		})
		.from(shopOrder)
		.innerJoin(shopItem, eq(shopOrder.itemId, shopItem.id))
		.where(eq(shopOrder.userId, userId))
		.orderBy(desc(shopOrder.createdAt));
}

export async function isUserAdmin(userId: string) {
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

export async function getAllShopOrders() {
	return db
		.select({
			id: shopOrder.id,
			status: shopOrder.status,
			pricePaid: shopOrder.pricePaid,
			notes: shopOrder.notes,
			fulfillmentMessage: shopOrder.fulfillmentMessage,
			createdAt: shopOrder.createdAt,
			fulfilledAt: shopOrder.fulfilledAt,
			itemName: shopItem.name,
			userName: user.name,
			userEmail: user.email
		})
		.from(shopOrder)
		.innerJoin(shopItem, eq(shopOrder.itemId, shopItem.id))
		.innerJoin(user, eq(shopOrder.userId, user.id))
		.orderBy(
			sql`CASE WHEN ${shopOrder.status} = 'in_queue' THEN 0 ELSE 1 END`,
			asc(shopOrder.createdAt)
		);
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
