FROM node:22-slim

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json index.js index.test.js ./

RUN ubt info
RUN ubt dep install
RUN ubt test
RUN ubt clean
