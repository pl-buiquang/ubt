FROM node:22-slim

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json index.test.js ./

# Create pnpm lockfile so variant is detected
RUN pnpm install
RUN ubt info
RUN ubt test
RUN ubt clean
