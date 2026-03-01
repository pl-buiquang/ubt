FROM ruby:3.3-slim

RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Gemfile hello.rb Rakefile ./
COPY test/ ./test/

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt dep lock
RUN ubt test
RUN ubt run-file hello.rb
