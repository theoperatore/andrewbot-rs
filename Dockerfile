# Builder stage
FROM rust:1.54.0 AS build
WORKDIR /app

COPY src/ ./src
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

FROM debian:buster-slim AS runtime

WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates libssl-dev \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/andrew-bot-rs andrew-bot-rs
USER 1000
ENTRYPOINT ["./andrew-bot-rs"]
