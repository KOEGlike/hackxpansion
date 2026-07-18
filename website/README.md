# HackXPansion Website

SvelteKit website for the landing page, documentation, authenticated project tracking, and Ari
review integration.

## Requirements

- Node.js 22.12 or newer
- PostgreSQL 18

## Development

Install dependencies and configure the variables documented in `.env.example`. Start the local
database and apply the current Drizzle schema:

```sh
npm install
npm run db:start
npm run db:push
```

Then start SvelteKit:

```sh
npm run dev
```

Use `npm run db:generate` when creating a new migration from the current schema. Commit the generated
SQL and metadata together.

## Production

The application requires a Node server for authentication, database-backed pages, form actions, and
Ari webhooks. It cannot be hosted as a static GitHub Pages site.

```sh
npm ci
npm run build
ORIGIN=https://example.com HOST=127.0.0.1 PORT=3000 npm start
```

Set `BASE_PATH` while building when the server is mounted below the origin root. Place a reverse
proxy in front of the Node process for TLS and configure it to forward the original host and protocol.

## Quality Checks

```sh
npm run check
npm run lint
npm test
```
