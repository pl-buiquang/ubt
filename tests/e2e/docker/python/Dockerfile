FROM python:3.12-slim

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY requirements.txt hello.py test_hello.py ./

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt lint.fix
RUN ubt lint
RUN ubt fmt
RUN ubt fmt --check
RUN ubt test
RUN ubt run-file hello.py
RUN ubt clean
