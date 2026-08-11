import { defineConfig } from 'drizzle-kit';

const databaseUrl = process.env.APP_DATABASE_URL || process.env.DATABASE_URL;
if (!databaseUrl) throw new Error('APP_DATABASE_URL or DATABASE_URL is not set');

export default defineConfig({
	schema: './src/lib/server/db/schema.ts',
	out: './drizzle',
	dialect: 'postgresql',
	dbCredentials: { url: databaseUrl },
	verbose: true,
	strict: true
});
