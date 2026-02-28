FROM python:3.12-slim

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY requirements.txt hello.py test_hello.py ./

RUN ubt info
RUN ubt dep install
RUN ubt test
RUN ubt clean
