FROM mcr.microsoft.com/dotnet/sdk:8.0

COPY ubt /usr/local/bin/ubt
WORKDIR /app
COPY Hello.csproj UnitTest.cs ./

RUN ubt info
RUN ubt dep install
RUN ubt dep list
RUN ubt build
RUN ubt test
RUN ubt fmt
RUN ubt clean
