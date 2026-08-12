const MAX_MESSAGE_BYTES = 2_000;
const MAX_STACK_LINES = 12;
const MAX_JSON_DEPTH = 20;
const MAX_JSON_ITEMS = 500;
const SENSITIVE_KEY =
	/(?:api[_-]?key|(?:access|refresh|id|client)[_-]?(?:token|secret)|token|secret|password|authorization|cookie|email|name|address|birthday)/i;

export type InternalErrorDetails = {
	name: string;
	message: string;
	stack?: string;
	cause?: InternalErrorDetails | InternalErrorDetails[];
};

export function internalErrorDetails(error: unknown, depth = 0): InternalErrorDetails {
	if (error instanceof Error) {
		const cause =
			depth >= 3
				? undefined
				: error instanceof AggregateError
					? error.errors.map((item) => internalErrorDetails(item, depth + 1))
					: error.cause
						? internalErrorDetails(error.cause, depth + 1)
						: undefined;
		return {
			name: error.name,
			message: sanitizeLogText(error.message),
			...(error.stack
				? {
						stack: sanitizeLogText(error.stack).split('\n').slice(0, MAX_STACK_LINES).join('\n')
					}
				: {}),
			...(cause ? { cause } : {})
		};
	}

	return {
		name: typeof error,
		message: sanitizeLogText(String(error))
	};
}

export function upstreamResponseExcerpt(responseBody: string | undefined) {
	if (!responseBody) return null;
	let sanitized: string;
	try {
		sanitized = JSON.stringify(redactJsonValue(JSON.parse(responseBody)));
	} catch {
		sanitized = sanitizeLogText(responseBody);
	}
	return {
		bytes: Buffer.byteLength(responseBody),
		excerpt: truncateUtf8(sanitized)
	};
}

function sanitizeLogText(value: string) {
	const withoutControlCharacters = Array.from(value, (character) => {
		const code = character.charCodeAt(0);
		return code < 32 && code !== 9 && code !== 10 && code !== 13 ? '' : character;
	}).join('');

	return withoutControlCharacters
		.replace(/([a-z][a-z0-9+.-]*:\/\/)[^\s/@]+(?::[^\s/@]*)?@/gi, '$1[redacted]@')
		.replace(/\bBearer\s+\S+/gi, 'Bearer [redacted]')
		.replace(/\b(?:pat\w*|xox[baprs]|whsec)_[A-Za-z0-9._-]+\b/g, '[redacted]')
		.replace(/\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b/gi, '[redacted-email]')
		.replace(
			/\b(api[_-]?key|(?:access|refresh|id|client)[_-]?(?:token|secret)|token|secret|password|authorization|cookie|email|name|address|birthday)\s*[:=]\s*[^\r\n]*/gi,
			'$1=[redacted]'
		)
		.replace(/\bU[A-Z0-9]{8,}\b/g, '[redacted-slack-id]')
		.replace(/\+?\d[\d(). -]{7,}\d/g, '[redacted-phone]')
		.replace(
			/\b\d{1,6}\s+[A-Za-z0-9.' -]{2,40}\s(?:Street|St|Road|Rd|Lane|Ln|Avenue|Ave|Boulevard|Blvd|Drive|Dr|Court|Ct)\b/gi,
			'[redacted-address]'
		);
}

function redactJsonValue(value: unknown, depth = 0, state = { items: 0 }): unknown {
	if (depth >= MAX_JSON_DEPTH || state.items >= MAX_JSON_ITEMS) return '[truncated]';
	state.items += 1;
	if (Array.isArray(value)) {
		return value.map((item) => redactJsonValue(item, depth + 1, state));
	}
	if (value && typeof value === 'object') {
		return Object.fromEntries(
			Object.entries(value).map(([key, item]) => [
				key,
				SENSITIVE_KEY.test(key) ? '[redacted]' : redactJsonValue(item, depth + 1, state)
			])
		);
	}
	return typeof value === 'string' ? sanitizeLogText(value) : value;
}

function truncateUtf8(value: string) {
	const bytes = Buffer.from(value);
	if (bytes.length <= MAX_MESSAGE_BYTES) return value;
	return bytes
		.subarray(0, MAX_MESSAGE_BYTES)
		.toString('utf8')
		.replace(/\uFFFD$/, '');
}
