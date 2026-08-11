# Hackxpansion Website

SvelteKit website for the landing page, documentation, authenticated project tracking, and Ari
review integration.

## Requirements

- Node.js 22.12 or newer
- PostgreSQL 18
- Docker (user needs to be added to docker user group)

## Development

Install dependencies and configure the variables documented in `.env.example`. Start the local
database and apply the current Drizzle schema:

```sh
npm install
npm run db:start
npm run db:migrate
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

Set `DATABASE_URL` to the managed PostgreSQL URL. If the deployment platform reserves or overrides
`DATABASE_URL`, set `APP_DATABASE_URL` instead; it takes precedence for migrations and the app.

Deploy production from the root `Dockerfile`. The container applies pending Drizzle migrations
before starting the server, serializing concurrent starts with a PostgreSQL advisory lock.

## Local Docker Compose

Configure `.env`, then build and start the app and local PostgreSQL database:

```sh
docker compose up --build -d
```

The application container applies pending Drizzle migrations before starting the server. Concurrent
starts are serialized with a PostgreSQL advisory lock. The application stack can be started with
`npm run compose:up`.

The app is served at `http://localhost:3000` by default. Set `APP_PORT` to change the host port:

```sh
APP_PORT=8080 docker compose up --build -d
```

Set `ORIGIN` to the public application URL in the runtime environment when deploying the Docker
image.

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
