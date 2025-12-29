# Build stage with cargo-chef for dependency caching
FROM rust:1.91-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev perl make gcc \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/bytehub .

EXPOSE 3000

CMD ["./bytehub"]
