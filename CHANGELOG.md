# Changelog

## Unreleased
- Implemented Phase 1 skeleton in Rust using Axum and Tokio.
- Added TOML and environment variable configuration loading defaulting to localhost port 9471.
- Implemented structured error model with OpenAI-compatible JSON error responses.
- Implemented `Provider` trait and deterministic `MockProvider` and `ProviderRegistry`.
- Implemented endpoints `GET /health`, `GET /v1/models`, and `POST /v1/chat/completions`.
- Added unit tests and integration tests with 100% test pass rate.
- Added repository overview, agent operating rules, phased plan, decision log, progress tracker, blockers list, and Jules audit brief.
