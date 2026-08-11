import { fileURLToPath } from 'node:url';
import { setTimeout } from 'node:timers/promises';
import { drizzle } from 'drizzle-orm/postgres-js';
import { migrate } from 'drizzle-orm/postgres-js/migrator';
import postgres from 'postgres';

const migrationLockId = 0x6861636b; // "hack"
const migrationsFolder = fileURLToPath(new URL('../drizzle', import.meta.url));
const lockTimeoutMs = 30_000;

console.log('Container startup: applying database migrations.');

try {
	await runMigrations();
} catch (error) {
	console.error('Container startup failed while applying database migrations:', error);
	process.exitCode = 1;
}

async function runMigrations() {
	const databaseUrl = process.env.APP_DATABASE_URL || process.env.DATABASE_URL;
	if (!databaseUrl) throw new Error('APP_DATABASE_URL or DATABASE_URL is not set');
	if (databaseUrl.includes('${')) {
		throw new Error('Database URL contains an unexpanded environment expression');
	}

	const client = postgres(databaseUrl, { max: 1, connect_timeout: 10 });
	let lockAcquired = false;

	try {
		const deadline = Date.now() + lockTimeoutMs;
		do {
			const [lock] = await client`
				SELECT pg_try_advisory_lock(${migrationLockId}) AS acquired
			`;
			lockAcquired = lock?.acquired === true;
			if (!lockAcquired) await setTimeout(1_000);
		} while (!lockAcquired && Date.now() < deadline);

		if (!lockAcquired) throw new Error('Timed out waiting for the database migration lock');

		await migrate(drizzle(client), { migrationsFolder });
		console.log('Database migrations applied successfully.');
	} finally {
		if (lockAcquired) {
			await client`SELECT pg_advisory_unlock(${migrationLockId})`.catch(() => undefined);
		}
		await client.end({ timeout: 5 });
	}
}
