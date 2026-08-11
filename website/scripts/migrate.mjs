import { fileURLToPath } from 'node:url';
import { drizzle } from 'drizzle-orm/postgres-js';
import { migrate } from 'drizzle-orm/postgres-js/migrator';
import postgres from 'postgres';

const databaseUrl = process.env.APP_DATABASE_URL || process.env.DATABASE_URL;
if (!databaseUrl) throw new Error('APP_DATABASE_URL or DATABASE_URL is not set');

const migrationLockId = 0x6861636b; // "hack"
const migrationsFolder = fileURLToPath(new URL('../drizzle', import.meta.url));
const client = postgres(databaseUrl, { max: 1 });

try {
	await client`SELECT pg_advisory_lock(${migrationLockId})`;
	await migrate(drizzle(client), { migrationsFolder });
	console.log('Database migrations applied successfully.');
} finally {
	await client`SELECT pg_advisory_unlock(${migrationLockId})`.catch(() => undefined);
	await client.end();
}
