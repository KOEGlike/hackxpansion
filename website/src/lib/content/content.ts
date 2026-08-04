export const landingContent = {
	hero: {
		title: 'Hackxpansion',
		subtitle: 'Design 4 expansion cards, build them for free, get a custom console to use them in!'
	},
	steps: [
		{
			title: 'Step 1',
			description:
				'Design your card! Make a weird input card, add a radio, a speaker or a motor! Go wild!'
		},
		{
			title: 'Step 2',
			description:
				"Code a driver in Rust! Don't worry, it won't be hard, we'll help you through it!"
		},
		{
			title: 'Step 3',
			description:
				"Repeat 3 more times! Make modules that complement each other, or don't! Be creative!"
		},
		{ title: 'Step 4', description: 'Submit and get the console!' }
	],
	simpleStepMedia: [
		{ imageSrc: '/simple/module.webp', imageAlt: 'one module', reversed: false },
		{ imageSrc: '/ferris.webp', imageAlt: 'ferris', reversed: true },
		{ imageSrc: '/simple/4modules.webp', imageAlt: 'four modules', reversed: false },
		{ imageSrc: '/simple/exploaded-console.webp', imageAlt: 'console', reversed: true }
	],
	faqs: [
		{
			question: 'What is Hackxpansion?',
			answer:
				'Hackxpansion is a Hack Club event where you make four expansion cards, write drivers and apps for them, and earn a custom handheld console that can use them.'
		},
		{
			question: 'Where do I get started?',
			answer:
				'Join the Hack Club Slack, visit the #hackxpansion channel, and follow the "Getting Started" guides in the documentation.'
		},
		{
			question: 'What if I am new to hardware?',
			answer:
				'That is okay. The guides are designed to help you make your first module or PCB, and the community can help when you get stuck.'
		},
		{
			question: 'How do I track my work?',
			answer:
				'You can journal your work on the platform, record a timelapse with Lapse, or track coding time with Hackatime. Do not report the same work through more than one method.'
		},
		{
			question: 'What if I do not have a 3D printer?',
			answer:
				'Join #printing-legion in the Hack Club Slack to have your models printed by the community printer network.'
		}
	]
} as const;
