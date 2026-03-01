FROM rust:1.83-slim-bookworm

RUN rustup component add rustfmt clippy

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt dep lock
RUN ubt check
RUN ubt fmt
RUN ubt fmt --check
RUN ubt lint
RUN ubt test
RUN ubt build
RUN ubt release
RUN ubt start
RUN ubt clean
