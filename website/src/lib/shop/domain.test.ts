import { describe, expect, it } from 'vitest';
import { getShopEligibility, HACKXPANSION_CONSOLE, isShopItemUnlocked } from './domain';

describe('shop eligibility', () => {
	it('requires four accepted module designs and one accepted app design for the console', () => {
		expect(HACKXPANSION_CONSOLE).toMatchObject({
			price: 8,
			requiredModuleDesigns: 4,
			requiredAppDesigns: 1
		});

		expect(getShopEligibility(HACKXPANSION_CONSOLE, { moduleDesigns: 3, appDesigns: 1 })).toEqual({
			eligible: false,
			missingModuleDesigns: 1,
			missingAppDesigns: 0
		});
		expect(getShopEligibility(HACKXPANSION_CONSOLE, { moduleDesigns: 4, appDesigns: 0 })).toEqual({
			eligible: false,
			missingModuleDesigns: 0,
			missingAppDesigns: 1
		});

		expect(getShopEligibility(HACKXPANSION_CONSOLE, { moduleDesigns: 4, appDesigns: 1 })).toEqual({
			eligible: true,
			missingModuleDesigns: 0,
			missingAppDesigns: 0
		});
	});

	it('keeps the console available and unlocks other items after a console order', () => {
		expect(isShopItemUnlocked(HACKXPANSION_CONSOLE.id, false)).toBe(true);
		expect(isShopItemUnlocked('accessory', false)).toBe(false);
		expect(isShopItemUnlocked('accessory', true)).toBe(true);
		expect(isShopItemUnlocked(HACKXPANSION_CONSOLE.id, true)).toBe(true);
	});
});
