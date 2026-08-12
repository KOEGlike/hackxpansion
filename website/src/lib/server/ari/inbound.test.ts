import { createHmac } from 'node:crypto';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	buildAriIngestPayload,
	sendAriIngest,
	sendAriWithdraw,
	type AriIngestPayload
} from './inbound';

const payload: AriIngestPayload = {
	external_id: 'project:design:delivery',
	title: 'Expansion card',
	description: 'A useful expansion card.',
	maker: { email: 'maker@example.com', name: 'Maker', slack_id: 'U123' },
	repo_url: 'https://github.com/hackclub/example',
	track: 'hardware',
	thumbnail_url: 'https://example.com/card.png',
	evidence: ['commits', 'elapsed', 'devlog'],
	meta: { 'Project ID': 'project' }
};

afterEach(() => vi.unstubAllGlobals());

describe('Ari ingest', () => {
	it('builds the documented solo ship payload', () => {
		const result = buildAriIngestPayload({
			externalId: 'project:design:delivery',
			project: {
				id: 'project',
				title: 'Expansion card',
				description: 'A useful expansion card.',
				repoUrl: 'https://github.com/hackclub/example',
				demoUrl: null,
				thumbnailUrl: 'https://example.com/card.png',
				hackatime_projects: ['card-firmware']
			},
			maker: { email: 'maker@example.com', name: 'Maker', slackId: 'U123' },
			journals: [
				{ createdAt: new Date('2026-07-30T12:00:00Z'), durationInMinutes: 45, text: 'Built it' }
			],
			phase: 'design'
		});

		expect(result).toMatchObject({
			external_id: 'project:design:delivery',
			track: 'hardware',
			hackatime_projects: ['card-firmware'],
			journals: [{ at: '2026-07-30', minutes: 45, text: 'Built it' }]
		});
		expect(typeof result.journals?.[0].minutes).toBe('number');
	});

	it('sends journals using Ari date, numeric minutes, and text fields', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValue(new Response('{"status":"accepted"}', { status: 202 }));
		vi.stubGlobal('fetch', fetchMock);
		const payloadWithJournals: AriIngestPayload = {
			...payload,
			journals: [
				{ at: '2026-06-01', minutes: 90, text: 'Tide tables parsed' },
				{ at: '2026-06-02', minutes: 45, text: 'Clock face render' }
			]
		};

		await sendAriIngest(payloadWithJournals, {
			programId: 'program-id',
			signingSecret: 'secret'
		});
		const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
		const sentPayload = JSON.parse(String(init.body));

		expect(sentPayload.journals).toEqual([
			{ at: '2026-06-01', minutes: 90, text: 'Tide tables parsed' },
			{ at: '2026-06-02', minutes: 45, text: 'Clock face render' }
		]);
		expect(typeof sentPayload.journals[0].minutes).toBe('number');
	});

	it('uses the webhook host and signs the exact request body', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValue(new Response('{"status":"accepted"}', { status: 202 }));
		vi.stubGlobal('fetch', fetchMock);

		const result = await sendAriIngest(payload, {
			programId: 'program-id',
			signingSecret: 'secret'
		});
		const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
		const rawBody = JSON.stringify(payload);

		expect(url).toBe('https://webhooks.ari.hackclub.com/api/ingest/program-id');
		expect(init.body).toBe(rawBody);
		expect(new Headers(init.headers).get('X-Ari-Signature')).toBe(
			createHmac('sha256', 'secret').update(rawBody).digest('hex')
		);
		expect(result).toEqual({ status: 202, body: '{"status":"accepted"}', duplicate: false });
	});

	it('recognizes a 200 retry as a duplicate', async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('duplicate', { status: 200 })));

		await expect(
			sendAriIngest(payload, { programId: 'program-id', signingSecret: 'secret' })
		).resolves.toMatchObject({ duplicate: true });
	});

	it('rejects Ari already-queued conflicts', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(new Response('already queued', { status: 409 }))
		);

		await expect(
			sendAriIngest(payload, { programId: 'program-id', signingSecret: 'secret' })
		).rejects.toMatchObject({ status: 409, responseBody: 'already queued' });
	});

	it('uses the documented withdrawal endpoint and body signature', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValue(new Response('{"status":"withdrawn"}', { status: 200 }));
		vi.stubGlobal('fetch', fetchMock);

		await sendAriWithdraw('project:design:delivery', {
			programId: 'program-id',
			signingSecret: 'secret'
		});
		const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
		const rawBody = JSON.stringify({ external_id: 'project:design:delivery' });

		expect(url).toBe('https://webhooks.ari.hackclub.com/api/ingest/program-id/withdraw');
		expect(new Headers(init.headers).get('X-Ari-Signature')).toBe(
			createHmac('sha256', 'secret').update(rawBody).digest('hex')
		);
	});
});
