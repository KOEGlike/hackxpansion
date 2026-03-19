export const landingContent = {
	hero: {
		title: 'Hackxpansion',
		subtitle: 'Make 4 expansion cards, get a custom console to use them in!'
	},
	steps: [
		{ title: 'Step 1', description: 'Design your card!' },
		{ title: 'Step 2', description: 'Code a driver!' },
		{ title: 'Step 3', description: 'Make 3 more!' },
		{ title: 'Step 4', description: 'Submit and get the console!' }
	],
	simpleStepMedia: [
		{ imageSrc: '/simple/module.webp', imageAlt: 'one module', reversed: false },
		{ imageSrc: '/ferris.webp', imageAlt: 'ferris', reversed: true },
		{ imageSrc: '/simple/4modules.webp', imageAlt: 'four modules', reversed: false },
		{ imageSrc: '/simple/exploaded-console.webp', imageAlt: 'console', reversed: true }
	]
} as const;
