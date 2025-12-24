# Build stage
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies (OpenSSL is required by sqlx and octocrab)
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/bytehub .
COPY --from=builder /usr/src/app/migrations ./migrations

# Expose the port (Northflank will map this)
EXPOSE 3000

CMD ["./bytehub"]
