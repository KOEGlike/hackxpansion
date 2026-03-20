export const landingContent = {
	hero: {
		title: 'Hackxpansion',
		subtitle: 'Make 4 expansion cards, get a custom console to use them in!'
	},
	steps: [
		{
			title: 'Step 1',
			description: 'Design your card! Make a weird input card, or add a radio! Go wild!'
		},
		{
			title: 'Step 2',
			description:
				"Code a driver in Rust! Don't worry, it won't be hard, we'll help you through it!"
		},
		{
			title: 'Step 3',
			description: "Repeat 3 times! Make modules that complement each other, or don't! Be creative!"
		},
		{ title: 'Step 4', description: 'Submit and get the console!' }
	],
	simpleStepMedia: [
		{ imageSrc: '/simple/module.webp', imageAlt: 'one module', reversed: false },
		{ imageSrc: '/ferris.webp', imageAlt: 'ferris', reversed: true },
		{ imageSrc: '/simple/4modules.webp', imageAlt: 'four modules', reversed: false },
		{ imageSrc: '/simple/exploaded-console.webp', imageAlt: 'console', reversed: true }
	]
} as const;
