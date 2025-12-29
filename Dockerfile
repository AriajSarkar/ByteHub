# Build stage with cargo-chef for dependency caching
FROM rust:1.91-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# Install build dependencies including perl for OpenSSL, Node.js for Convex
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev perl make gcc curl \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and install Node dependencies
COPY . .
RUN pnpm install

# Build args for Convex and Discord (passed at build time)
ARG CONVEX_URL
ARG CONVEX_DEPLOYMENT
ARG ENVIRONMENT=dev
ARG DISCORD_BOT_TOKEN
ARG DISCORD_APPLICATION_ID

# Sync Convex functions (dev uses dev --once, prod uses deploy)
RUN if [ "$ENVIRONMENT" = "prod" ]; then \
    CONVEX_URL=$CONVEX_URL CONVEX_DEPLOYMENT=$CONVEX_DEPLOYMENT npx convex deploy -y; \
    else \
    CONVEX_URL=$CONVEX_URL CONVEX_DEPLOYMENT=$CONVEX_DEPLOYMENT npx convex dev --once; \
    fi

# Register Discord commands (build the binary first, then run it)
RUN cargo build --release --bin register_commands
RUN DISCORD_BOT_TOKEN=$DISCORD_BOT_TOKEN \
    DISCORD_APPLICATION_ID=$DISCORD_APPLICATION_ID \
    ./target/release/register_commands

# Build the main application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/bytehub .

EXPOSE 3000

CMD ["./bytehub"]
