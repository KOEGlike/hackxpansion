const DEFAULT_TIMEOUT_MS = 10_000;

export function fetchWithTimeout(
	input: string | URL | Request,
	init: RequestInit = {},
	timeoutMs = DEFAULT_TIMEOUT_MS
) {
	const timeoutSignal = AbortSignal.timeout(timeoutMs);
	const signal = init.signal ? AbortSignal.any([init.signal, timeoutSignal]) : timeoutSignal;
	return fetch(input, { ...init, signal });
}
