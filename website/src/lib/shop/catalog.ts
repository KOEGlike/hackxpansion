import { HACKXPANSION_CONSOLE } from './domain';

const MAX_TEXT_LENGTH = 2_000;
const MAX_NAME_LENGTH = 100;
const MAX_ID_LENGTH = 100;
const POSTGRES_INTEGER_MAX = 2_147_483_647;
const ITEM_ID_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

export type CatalogItemFormValues = {
	id: string;
	name: string;
	description: string;
	price: string;
	imageUrl: string;
	sortOrder: string;
	active: boolean;
};

export type CatalogItemInput = {
	id: string;
	name: string;
	description: string;
	price: number;
	imageUrl: string | null;
	sortOrder: number;
	active: boolean;
};

export class CatalogItemValidationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = 'CatalogItemValidationError';
	}
}

export function parseCatalogItem(values: CatalogItemFormValues): CatalogItemInput {
	const id = values.id.trim();
	const name = values.name.trim();
	const description = values.description.trim();
	const imageUrl = values.imageUrl.trim();

	if (!id || id.length > MAX_ID_LENGTH || !ITEM_ID_PATTERN.test(id)) {
		throw new CatalogItemValidationError(
			'Item ID must be 1-100 lowercase letters, numbers, or single hyphens.'
		);
	}
	if (id === HACKXPANSION_CONSOLE.id) {
		throw new CatalogItemValidationError('The Hackxpansion Console is managed in code.');
	}
	if (!name || name.length > MAX_NAME_LENGTH) {
		throw new CatalogItemValidationError(`Name must be 1-${MAX_NAME_LENGTH} characters.`);
	}
	if (!description || description.length > MAX_TEXT_LENGTH) {
		throw new CatalogItemValidationError(`Description must be 1-${MAX_TEXT_LENGTH} characters.`);
	}
	if (!/^\d+$/.test(values.price)) {
		throw new CatalogItemValidationError('Price must be a non-negative whole number.');
	}
	const price = Number(values.price);
	if (!Number.isSafeInteger(price) || price > POSTGRES_INTEGER_MAX) {
		throw new CatalogItemValidationError('Price is too large.');
	}
	if (!/^-?\d+$/.test(values.sortOrder)) {
		throw new CatalogItemValidationError('Sort order must be a whole number.');
	}
	const sortOrder = Number(values.sortOrder);
	if (!Number.isSafeInteger(sortOrder) || Math.abs(sortOrder) > POSTGRES_INTEGER_MAX) {
		throw new CatalogItemValidationError('Sort order is too large.');
	}
	if (imageUrl.length > MAX_TEXT_LENGTH || (imageUrl && !isValidImageUrl(imageUrl))) {
		throw new CatalogItemValidationError(
			'Image URL must be an http(s) URL or a root-relative path beginning with /.'
		);
	}

	return {
		id,
		name,
		description,
		price,
		imageUrl: imageUrl || null,
		sortOrder,
		active: values.active
	};
}

function isValidImageUrl(value: string) {
	if (value.startsWith('/') && !value.startsWith('//')) return true;
	try {
		return ['http:', 'https:'].includes(new URL(value).protocol);
	} catch {
		return false;
	}
}
