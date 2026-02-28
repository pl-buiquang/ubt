FROM rust:1.83-slim-bookworm

RUN rustup component add rustfmt clippy

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

RUN ubt info
RUN ubt test
RUN ubt build
RUN ubt fmt --check
RUN ubt lint
RUN ubt clean
