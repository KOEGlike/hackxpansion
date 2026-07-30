import { createHmac } from 'node:crypto';
import { describe, expect, it } from 'vitest';
import { processOutboundRequest } from './outbound';

function signedRequest(body: object, secret: string, deliveryId = 'delivery-1') {
	const timestamp = String(Math.floor(Date.now() / 1000));
	const rawBody = JSON.stringify(body);
	const signature = createHmac('sha256', secret)
		.update(`${timestamp}.${deliveryId}.${rawBody}`)
		.digest('hex');

	return new Request('https://example.com/hooks/ari', {
		method: 'POST',
		headers: {
			'X-Ari-Signature': signature,
			'X-Ari-Timestamp': timestamp,
			'X-Ari-Delivery-Id': deliveryId,
			'Content-Type': 'application/json'
		},
		body: rawBody
	});
}

describe('Ari outcome webhooks', () => {
	it('verifies and parses the updated approved payload', async () => {
		const body = {
			event: 'review.approved',
			decision: 'approved',
			id: 'AR-4821',
			external_id: 'project:design:delivery',
			maker: { email: 'maker@example.com', slack_id: 'U123' },
			review: {
				approved_minutes: 120,
				minutes_breakdown: { hackatime: 60, journals: 30, lapse: 0, program: 30 },
				justification: { technical_features: 'Custom protocol and PCB design' },
				note_to_maker: 'Approved'
			}
		};

		const result = await processOutboundRequest(signedRequest(body, 'secret'), 'secret');

		expect(result.headers.delivery_id).toBe('delivery-1');
		expect(result.body).toEqual(body);
	});

	it('rejects an invalid signature', async () => {
		const request = signedRequest(
			{
				event: 'review.requeued',
				decision: null,
				id: 'AR-4821',
				external_id: 'project:design:delivery',
				maker: { email: 'maker@example.com', slack_id: 'U123' },
				review: {}
			},
			'wrong-secret'
		);

		await expect(processOutboundRequest(request, 'secret')).rejects.toMatchObject({
			status: 401
		});
	});
});
