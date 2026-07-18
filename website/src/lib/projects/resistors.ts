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
	md1: ModuleResistor;
	md2: ModuleResistor;
};

export function isModuleResistor(value: number): value is ModuleResistor {
	return (E24_RESISTOR_VALUES as readonly number[]).includes(value);
}

export function formatResistor(ohms: number): string {
	if (ohms >= 1000) {
		const kilo = ohms / 1000;
		return `${kilo}k`;
	}
	return `${ohms}`;
}

export function findNextAvailableResistorPair(
	used: Array<{ md1: number | null; md2: number | null }>
): ModuleResistorPair | null {
	const usedSet = new Set(
		used
			.filter((pair): pair is { md1: number; md2: number } => pair.md1 != null && pair.md2 != null)
			.map((pair) => `${pair.md1}:${pair.md2}`)
	);

	for (const md1 of E24_RESISTOR_VALUES) {
		for (const md2 of E24_RESISTOR_VALUES) {
			if (!usedSet.has(`${md1}:${md2}`)) {
				return { md1, md2 };
			}
		}
	}

	return null;
}
