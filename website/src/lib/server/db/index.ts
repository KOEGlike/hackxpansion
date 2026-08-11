import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';
import { env } from '$env/dynamic/private';

const databaseUrl = env.APP_DATABASE_URL || env.DATABASE_URL;
if (!databaseUrl) throw new Error('APP_DATABASE_URL or DATABASE_URL is not set');

const client = postgres(databaseUrl);

export const db = drizzle(client, { schema });
