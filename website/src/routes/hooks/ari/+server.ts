import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { OutboundBody } from '$lib/server/ari/outbound';
import { createHmac } from 'crypto';
import { env } from 'process';

export const POST: RequestHandler = async ({ request }) => {
	if (!env.ARI_OUT_SECRET) {
		error(500, 'ARI_OUT_SECRET environment variable is not set');
	}

	const signature = request.headers.get('X-Ari-Signature');
	const timestamp = request.headers.get('X-Ari-Timestamp');
	const deliverId = request.headers.get('X-Ari-Deliver-Id');

	if (!signature || !timestamp || !deliverId) {
		error(400, 'X-Ari-Signature, X-Ari-Timestamp, and X-Ari-Deliver-Id headers are required');
	}

	const calculated_signature = createHmac('sha256', env.ARI_OUT_SECRET)
		.update(await request.bytes())
		.digest('hex');

	if (signature !== calculated_signature) {
		error(401, 'Invalid signature');
	}

	const body: OutboundBody = await request.json();

	return new Response(null, { status: 200 });
};
