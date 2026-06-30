export const eventTypeArray = [
	'approved',
	'changes',
	'rejected',
	'reverted',
	'requeued',
	'fraud'
] as const;

export type ReviewEventType = (typeof eventTypeArray)[number];

export type Event = `review.${ReviewEventType}`;

export type OutboundHeaders = {
	signature: string;
	timestamp: string;
	delivery_id: string;
};

type MinutesBreakdown = {
	hackatime: number;
	journal: number;
	lapse: number;
	progress: number;
};

export type Reviewer = {
	email: string;
	slack_id: string;
};

export type FraudReview = {
	verdict: string;
	checks: [
		{
			email: string;
			slack_id: string;
			trust_score: number;
			justification: string;
		}
	];
};

export type OutboundBody = {
	event: Event;
	decision?: string;
	id: string;
	external_id: string;
	maker: {
		email: string;
		slack_id: string;
	};
	collaborators: {
		email: string;
		name: string;
		slack_id: string;
		hackatime_id: string;
		approved_minutes: number;
		approved_hours: number;
		minutes_breakdown: MinutesBreakdown;
	}[];
	review: {
		approved_minutes: number;
		approved_hours: number;
		minutes_breakdown: MinutesBreakdown;
		note_to_maker: string;
		audit_note: string;
		fields: {
			key: string;
			label: string;
			type: string;
			value: string;
		}[];
		reviewer: Reviewer;
	};
	fraud?: FraudReview;
};

export function fromOutboundEvent(event: Event): ReviewEventType {
	return event.replace(/^review\./, '') as ReviewEventType;
}

export function processOutboundRequest(request: Request): Promise<OutboundBody> {
	
}
