# Contributing to ByteHub

> ⚠️ **IMPORTANT: By contributing to this project, you agree to the [Contributor License Agreement](LICENSE.md#8-contributor-license-agreement).**
>
> Opening a pull request means you accept that your contributions become the property of the project maintainer with no claims to royalties or ownership.

Welcome! This guide outlines how to contribute to ByteHub while maintaining our high standards for reliability and testing.

## Tech Stack

- **Language**: Rust (Stable)
- **Web Framework**: Axum
- **Database**: [Convex](https://convex.dev) (serverless backend)
- **Discord Library**: twilight-rs (twilight-http, twilight-model)
- **GitHub Integration**: octocrab

## Project Structure

We use a **Library + Binary** structure:

```
src/
├── lib.rs              # Core logic and application state
├── main.rs             # Production server entry point
├── config.rs           # Environment configuration
├── error.rs            # Error types
├── discord/
│   ├── client.rs       # Discord API client
│   ├── commands.rs     # Slash command handlers
│   ├── rate_limit.rs   # Per-guild rate limiting
│   └── verify.rs       # Signature verification
├── github/
│   ├── events.rs       # Webhook event parsing
│   └── webhook.rs      # Webhook handler
├── governance/
│   ├── projects.rs     # Project management
│   ├── server_config.rs# Server configuration
│   └── rules.rs        # Event routing rules
├── router/
│   └── dispatch.rs     # Event dispatching logic
└── storage/
    └── convex.rs       # Convex database client

convex/
├── schema.ts           # Database schema
├── projects.ts         # Project mutations/queries
├── serverConfig.ts     # Server config mutations/queries
└── rules.ts            # Rules mutations/queries

tests/
├── github/             # GitHub event integration tests
├── discord/            # Discord interaction tests
└── common/             # Shared test utilities
```

## Development Setup

1. Copy `.env.example` to `.env` and fill in your credentials
2. Start Convex dev server: `npx convex dev`
3. Register Discord commands: `cargo run --bin register_commands`
4. Run the server: `cargo run`

## Testing Strategy

We prioritize **event-driven logic tests** to ensure the bot triages GitHub events correctly.

### Running Tests

```bash
# Ensure CONVEX_URL is set in .env
cargo test
```

### Test Categories

- `tests/github/` - GitHub webhook event handling
- `tests/discord/` - Discord command interactions
- `tests/common/` - Mock Discord client for testing

## Security Features

### Rate Limiting

Expensive commands (`/setup-server`, `/approve`) are rate-limited to **5 requests per 60 seconds per guild** to prevent:
- Database write conflicts
- Command spam abuse
- Resource exhaustion

See `src/discord/rate_limit.rs` for implementation.

### Guild-Only Commands

All commands have `dm_permission: false` and `contexts: [0]` to ensure they only work in servers, not DMs.

### Idempotent Database Operations

Convex mutations check for data changes before writing to prevent write conflicts during concurrent operations.

## Modular Development

When adding new features:

1. **New GitHub Events**: Update `src/github/events.rs` with structures, add dispatch logic in `src/router/dispatch.rs`
2. **New Discord Commands**: Add handler in `src/discord/commands.rs`, register in `src/bin/register_commands.rs`
3. **New Database Tables**: Update `convex/schema.ts` and add corresponding TypeScript functions
4. **Always** add integration tests for new functionality

## Pull Request Guidelines

1. Run `cargo fmt` and `cargo clippy` before submitting
2. Ensure all tests pass: `cargo test`
3. Update documentation if adding new features
4. Keep PRs focused on a single feature or fix

Thank you for helping us keep ByteHub rock-solid!
