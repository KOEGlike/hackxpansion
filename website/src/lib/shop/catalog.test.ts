import { describe, expect, it } from 'vitest';
import {
	CatalogItemValidationError,
	parseCatalogItem,
	type CatalogItemFormValues
} from './catalog';

const validItem: CatalogItemFormValues = {
	id: 'usb-cable',
	name: 'USB Cable',
	description: 'A useful cable.',
	price: '2',
	imageUrl: '/shop/usb-cable.webp',
	sortOrder: '10',
	active: true
};

describe('shop catalog item validation', () => {
	it('normalizes a valid item', () => {
		expect(parseCatalogItem(validItem)).toEqual({
			id: 'usb-cable',
			name: 'USB Cable',
			description: 'A useful cable.',
			price: 2,
			imageUrl: '/shop/usb-cable.webp',
			sortOrder: 10,
			active: true
		});
	});

	it.each([
		['the hardcoded console', { id: 'hackxpansion-console' }],
		['an invalid ID', { id: 'USB Cable' }],
		['a fractional price', { price: '1.5' }],
		['an unsafe image URL', { imageUrl: 'javascript:alert(1)' }]
	])('rejects %s', (_case, changes) => {
		expect(() => parseCatalogItem({ ...validItem, ...changes })).toThrow(
			CatalogItemValidationError
		);
	});
});
