FROM ruby:3.3-slim

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Gemfile hello.rb ./
COPY test/ ./test/

RUN ubt info
RUN ubt dep install
RUN ubt clean
