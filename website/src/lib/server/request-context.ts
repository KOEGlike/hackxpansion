import { AsyncLocalStorage } from 'node:async_hooks';

const requestContext = new AsyncLocalStorage<{ requestId: string }>();

export function withRequestContext<T>(requestId: string, callback: () => T): T {
	return requestContext.run({ requestId }, callback);
}

export function currentRequestId() {
	return requestContext.getStore()?.requestId;
}
