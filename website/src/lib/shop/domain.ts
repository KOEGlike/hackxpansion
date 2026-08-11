export type ShopRequirements = {
	requiredModuleDesigns: number;
	requiredAppDesigns: number;
};

export type ShopProgress = {
	moduleDesigns: number;
	appDesigns: number;
};

export const HACKXPANSION_CONSOLE = {
	id: 'hackxpansion-console',
	name: 'Hackxpansion Console',
	description:
		'The main Hackxpansion prize: a console built to bring your modules and apps together.',
	price: 7,
	imageUrl: '/shop/console.png',
	requiredModuleDesigns: 4,
	requiredAppDesigns: 1,
	active: true,
	sortOrder: 0
} as const;

export function isShopItemUnlocked(itemId: string, hasConsoleOrder: boolean) {
	return itemId === HACKXPANSION_CONSOLE.id || hasConsoleOrder;
}

export function getShopEligibility(requirements: ShopRequirements, progress: ShopProgress) {
	const missingModuleDesigns = Math.max(
		0,
		requirements.requiredModuleDesigns - progress.moduleDesigns
	);
	const missingAppDesigns = Math.max(0, requirements.requiredAppDesigns - progress.appDesigns);

	return {
		eligible: missingModuleDesigns === 0 && missingAppDesigns === 0,
		missingModuleDesigns,
		missingAppDesigns
	};
}
