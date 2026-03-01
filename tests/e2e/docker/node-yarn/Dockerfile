FROM node:22-slim

RUN corepack enable && corepack prepare yarn@1.22.22 --activate

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json index.test.js ./

# Create yarn.lock so variant is detected
RUN yarn install
RUN ubt info
RUN ubt dep list
RUN ubt dep audit
RUN ubt build
RUN ubt test
RUN ubt lint
RUN ubt fmt
RUN ubt fmt --check
RUN ubt clean
