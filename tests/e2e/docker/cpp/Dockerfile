FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends cmake g++ && rm -rf /var/lib/apt/lists/*

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY CMakeLists.txt main.cpp ./

# Configure the build directory (prerequisite for cmake --build)
RUN cmake -B build

RUN ubt info
RUN ubt build
RUN ubt test
RUN ubt clean
