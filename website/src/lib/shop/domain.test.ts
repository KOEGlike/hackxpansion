import { describe, expect, it } from 'vitest';
import { getShopEligibility } from './domain';

describe('shop eligibility', () => {
	it('requires four accepted module designs and one accepted app design for the console', () => {
		expect(
			getShopEligibility(
				{ requiredModuleDesigns: 4, requiredAppDesigns: 1 },
				{ moduleDesigns: 3, appDesigns: 1 }
			)
		).toEqual({ eligible: false, missingModuleDesigns: 1, missingAppDesigns: 0 });

		expect(
			getShopEligibility(
				{ requiredModuleDesigns: 4, requiredAppDesigns: 1 },
				{ moduleDesigns: 4, appDesigns: 1 }
			)
		).toEqual({ eligible: true, missingModuleDesigns: 0, missingAppDesigns: 0 });
	});
});
