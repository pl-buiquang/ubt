FROM maven:3-eclipse-temurin-21

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY pom.xml ./
COPY src/ ./src/

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt build
RUN ubt test
RUN ubt clean
