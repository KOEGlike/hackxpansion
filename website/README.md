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

### Docker Compose

Configure `.env`, including `POSTGRES_PASSWORD`, then build and start the stack:

```sh
docker compose up --build -d
```

For a Portainer Git stack, configure the variables from `.env.example` in Portainer's stack
environment instead. The ignored local `.env` file is not required inside the deployment checkout.

The one-shot `migrate` service applies pending Drizzle migrations after PostgreSQL becomes healthy.
The app starts only after migration succeeds. Migrations can also be run manually with
`npm run compose:migrate`, and the full stack can be started with `npm run compose:up`.

The app is served at `http://localhost:3000` by default. Set `APP_PORT` to change the host port and
`COMPOSE_ORIGIN` to the app's public origin, for example:

```sh
APP_PORT=8080 COMPOSE_ORIGIN=https://example.com docker compose up --build -d
```

Generate and commit a new migration with `npm run db:generate` whenever the schema changes.

The initial migration expects a fresh database. A database previously initialized with `db:push`
has no Drizzle migration history; recreate that database or baseline it before enabling automatic
migrations if its existing data must be preserved.

## Quality Checks

```sh
npm run check
npm run lint
npm test
```
