FROM golang:1.23-bookworm

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY go.mod main.go main_test.go ./

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt dep lock
RUN ubt test
RUN ubt fmt
RUN ubt fmt --check
RUN ubt build
RUN ubt start
RUN ubt clean
