FROM rust:1.56.1 as build
WORKDIR /app

ENV RUST_ENV=production

COPY src/ ./src
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates libssl-dev \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/giantbomb_rs giantbomb-rs
USER 1000
EXPOSE 8080
ENTRYPOINT ["./giantbomb-rs"]
