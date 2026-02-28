FROM ruby:3.3-slim

RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Gemfile hello.rb ./
COPY test/ ./test/

RUN ubt info
RUN ubt dep install
