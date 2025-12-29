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
    && apt install nodejs -y \
    && npm install -g pnpm \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and install Node dependencies
COPY . .
RUN pnpm install

# Build the register_commands binary first
RUN cargo build --release --bin register_commands

# Deploy Convex functions using BuildKit secrets (never stored in image layers)
# Build with: --secret id=convex_key,env=CONVEX_DEPLOY_KEY
RUN --mount=type=secret,id=convex_key \
    CONVEX_DEPLOY_KEY=$(cat /run/secrets/convex_key) npx convex deploy

# Register Discord commands using BuildKit secrets
# Build with: --secret id=discord_token,env=DISCORD_BOT_TOKEN --secret id=discord_app_id,env=DISCORD_APPLICATION_ID
RUN --mount=type=secret,id=discord_token \
    --mount=type=secret,id=discord_app_id \
    DISCORD_BOT_TOKEN=$(cat /run/secrets/discord_token) \
    DISCORD_APPLICATION_ID=$(cat /run/secrets/discord_app_id) \
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
