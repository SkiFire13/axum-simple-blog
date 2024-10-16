FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Prepare the dependencies recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build the dependencies
# This will be cached by docker even when /src changes, which is great for development speed
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Build the application, this should be fast now that all dependencies are already built
COPY . .
RUN cargo build --release

# Finally, run the application
FROM docker.io/debian:bookworm-slim AS runtime
# Ensure that OpenSSL and root certificates are installed, they are needed to load avatar images
RUN apt-get update -y && apt-get install -y openssl ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/simple-blog /app

ENTRYPOINT ["/app/simple-blog"]
