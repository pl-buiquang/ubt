FROM golang:1.23-bookworm

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY go.mod main.go main_test.go ./

RUN ubt info
RUN ubt test
RUN ubt build
RUN ubt fmt
RUN ubt clean
