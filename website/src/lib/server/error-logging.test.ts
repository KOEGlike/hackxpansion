import { describe, expect, it } from 'vitest';
import { internalErrorDetails, upstreamResponseExcerpt } from './error-logging';

describe('server error logging', () => {
	it('redacts URL credentials and service tokens', () => {
		const details = internalErrorDetails(
			new Error('postgres://user:secret@db.example/test Bearer token pat_example_secret')
		);

		expect(details.message).toBe(
			'postgres://[redacted]@db.example/test Bearer [redacted] [redacted]'
		);
	});

	it('bounds upstream response bodies', () => {
		expect(upstreamResponseExcerpt(`whsec_secret ${'x'.repeat(3_000)}`)).toEqual({
			bytes: 3_013,
			excerpt: `[redacted] ${'x'.repeat(1_989)}`
		});
	});

	it('redacts common PII fields and email addresses from upstream responses', () => {
		expect(
			upstreamResponseExcerpt(
				'{"message":"maker ada@example.com failed","address":"1 Secret Lane","api_key":"key-1"}'
			)
		).toEqual({
			bytes: 86,
			excerpt:
				'{"message":"maker [redacted-email] failed","address":"[redacted]","api_key":"[redacted]"}'
		});
	});

	it('redacts escaped JSON and unquoted plain-text secrets', () => {
		expect(upstreamResponseExcerpt('{"secret":"abc\\"still-secret","code":"failed"}')).toEqual({
			bytes: 46,
			excerpt: '{"secret":"[redacted]","code":"failed"}'
		});
		expect(upstreamResponseExcerpt('failed token=plain-secret for U123456789')).toEqual({
			bytes: 40,
			excerpt: 'failed token=[redacted]'
		});
		expect(upstreamResponseExcerpt('client_secret=plain secret\nnext line')).toEqual({
			bytes: 36,
			excerpt: 'client_secret=[redacted]\nnext line'
		});
	});

	it('caps excerpts at 2000 UTF-8 bytes', () => {
		const result = upstreamResponseExcerpt('界'.repeat(2_000));
		expect(Buffer.byteLength(result!.excerpt)).toBeLessThanOrEqual(2_000);
	});

	it('bounds deeply nested JSON traversal', () => {
		const response = `${'['.repeat(5_000)}"leaf"${']'.repeat(5_000)}`;

		expect(() => upstreamResponseExcerpt(response)).not.toThrow();
		expect(upstreamResponseExcerpt(response)?.excerpt).toContain('[truncated]');
	});

	it('retains bounded nested error causes', () => {
		const error = new Error('Submission failed', {
			cause: new Error('fetch failed', { cause: new Error('ECONNREFUSED') })
		});

		expect(internalErrorDetails(error)).toMatchObject({
			message: 'Submission failed',
			cause: {
				message: 'fetch failed',
				cause: { message: 'ECONNREFUSED' }
			}
		});
	});
});
