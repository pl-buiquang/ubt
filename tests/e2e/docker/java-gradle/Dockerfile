FROM gradle:8-jdk21

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY build.gradle settings.gradle ./
COPY src/ ./src/

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt build
RUN ubt test
RUN ubt clean
