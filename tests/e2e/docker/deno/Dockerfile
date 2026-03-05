FROM denoland/deno:latest

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY deno.json main_test.ts ./

RUN ubt info | grep deno
RUN ubt dep install
RUN ubt test
RUN ubt fmt --check
RUN ubt lint
RUN ubt clean
