export type ShopRequirements = {
	requiredModuleDesigns: number;
	requiredAppDesigns: number;
};

export type ShopProgress = {
	moduleDesigns: number;
	appDesigns: number;
};

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
