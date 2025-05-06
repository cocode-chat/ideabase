# builder
FROM rust:1.86 AS builder

# copy cargo config
COPY .cargo/config.toml /usr/local/cargo/

# build and install
WORKDIR /usr/src/app
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .

# runtime
FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y tzdata \
#  && apt-get install -y default-libmysqlclient-dev \
  && rm -rf /var/lib/apt/lists/*

COPY yaml/* /app/yaml/
COPY --from=builder /usr/local/cargo/bin/ideabase /app/ideabase

ENV YML_DIR=/app/yaml

CMD ["/app/ideabase"]