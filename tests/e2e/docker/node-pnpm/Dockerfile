FROM node:22-slim

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json index.test.js ./

# Create pnpm lockfile so variant is detected
RUN pnpm install
RUN ubt info
RUN ubt dep list
RUN ubt dep audit
RUN ubt build
RUN ubt test
RUN ubt lint
RUN ubt fmt
RUN ubt fmt --check
RUN ubt clean
