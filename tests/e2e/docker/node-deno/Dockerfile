FROM denoland/deno:latest

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY package.json deno.json hello_test.ts ./

RUN ubt info
RUN ubt dep install
RUN ubt test
RUN ubt clean
