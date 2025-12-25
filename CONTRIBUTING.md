# Contributing to ByteHub

Welcome! This guide outlines how to contribute to ByteHub while maintaining our high standards for reliability and testing.

## Tech Stack
- **Language**: Rust (Stable)
- **Web Framework**: Axum
- **Database**: PostgreSQL (Aiven)
- **Database Toolkit**: sqlx (with offline/test macros)
- **Discord Library**: twilight-rs

## Project Structure
We use a **Library + Binary** structure:
- `src/lib.rs`: Contains core logic, models, and application state.
- `src/main.rs`: Lightweight entry point for the production server.
- `tests/github/`: Modular integration tests for each event type.

## Testing Strategy
We prioritize **event-driven logic tests** to ensure the bot triages GitHub events correctly. To prevent regressions, every major event should have a corresponding test in `tests/`.

### Database Isolation
We use `sqlx-test` for all integration tests. This ensures:
1. Every test runs against a **fresh, isolated logical database**.
2. Databases are automatically migrated and then dropped after the test.
3. No data collisions occur, even when tests run in parallel.

### Running Tests
Ensure your `.env` has a valid `DATABASE_URL` (ideally with `avnadmin` or a user with database creation rights).

```bash
cargo test
```

## Modular Development
When adding new GitHub events or triage rules:
1. Update `src/github/events.rs` with the new event structures.
2. Implement triage logic in `src/router/dispatch.rs`.
3. **Always** add a corresponding integration test in `tests/github_[event_type].rs`.

Thank you for helping us keep ByteHub rock-solid!
