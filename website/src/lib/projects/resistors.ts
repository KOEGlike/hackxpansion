export const E24_BASE_VALUES = [
	10, 11, 12, 13, 15, 16, 18, 20, 22, 24, 27, 30, 33, 36, 39, 43, 47, 51, 56, 62, 68, 75, 82, 91
] as const;

export const E24_RESISTOR_VALUES = [
	...E24_BASE_VALUES.map((v) => v * 100),
	...E24_BASE_VALUES.map((v) => v * 1000),
	100000
] as const;

export type ModuleResistor = (typeof E24_RESISTOR_VALUES)[number];

export type ModuleResistorPair = {
	md0: ModuleResistor;
	md1: ModuleResistor;
};

export function isModuleResistor(value: number): value is ModuleResistor {
	return (E24_RESISTOR_VALUES as readonly number[]).includes(value);
}

export function formatResistor(ohms: number): string {
	if (ohms >= 1000) {
		const kilo = ohms / 1000;
		return Number.isInteger(kilo) ? `${kilo}k` : `${kilo}`.replace('.', 'k');
	}
	return `${ohms}`;
}

export function findNextAvailableResistorPair(
	used: Array<{ md0: number | null; md1: number | null }>
): ModuleResistorPair | null {
	const usedSet = new Set(
		used
			.filter((pair): pair is { md0: number; md1: number } => pair.md0 != null && pair.md1 != null)
			.map((pair) => `${pair.md0}:${pair.md1}`)
	);

	for (const md0 of E24_RESISTOR_VALUES) {
		for (const md1 of E24_RESISTOR_VALUES) {
			if (!usedSet.has(`${md0}:${md1}`)) {
				return { md0, md1 };
			}
		}
	}

	return null;
}
