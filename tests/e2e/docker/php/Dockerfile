FROM composer:2

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY composer.json ./

RUN ubt info
RUN ubt dep install
RUN ubt test
RUN ubt build
RUN ubt clean
