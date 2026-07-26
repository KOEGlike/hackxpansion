FROM node:22-bookworm-slim AS dependencies

WORKDIR /app

COPY package.json package-lock.json svelte.config.js ./
RUN npm ci

FROM dependencies AS source

COPY . .

FROM source AS schema

USER node
CMD ["npm", "run", "db:push"]

FROM source AS build

ARG BASE_PATH=""
ENV BASE_PATH=$BASE_PATH

RUN DATABASE_URL=postgres://build:build@localhost/build \
	ORIGIN=http://localhost:3000 \
	BETTER_AUTH_SECRET=build-only-placeholder-secret-32-chars \
	HACKCLUB_CLIENT_ID=build \
	HACKCLUB_CLIENT_SECRET=build \
	npm run build && npm prune --omit=dev

FROM node:22-bookworm-slim AS runtime

WORKDIR /app

ENV NODE_ENV=production \
	HOST=0.0.0.0 \
	PORT=3000

COPY --from=build --chown=node:node /app/build ./build
COPY --from=build --chown=node:node /app/node_modules ./node_modules

USER node
EXPOSE 3000

CMD ["node", "build"]
