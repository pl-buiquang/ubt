FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends cmake g++ make && rm -rf /var/lib/apt/lists/*

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY CMakeLists.txt ./

# Inline the source so no .cpp file lives in the repo (avoids SonarCloud CFamily trigger)
RUN printf '#include <iostream>\nint main() { std::cout << "hello" << std::endl; return 0; }\n' > main.cpp

# Configure the build directory (prerequisite for cmake --build)
RUN cmake -B build

RUN ubt info
RUN ubt build
RUN ubt test
RUN ubt clean
