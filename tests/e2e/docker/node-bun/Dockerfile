FROM oven/bun:latest

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json index.test.js ./

# Create bun.lockb so variant is detected
RUN bun install
RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt dep audit
RUN ubt build
RUN ubt test
RUN ubt lint
RUN ubt fmt
RUN ubt fmt --check
RUN ubt clean
