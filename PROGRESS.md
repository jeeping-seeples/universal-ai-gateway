# Progress

## Current status

- [x] Repository purpose and boundaries documented.
- [x] Human-steered agent workflow documented.
- [x] Read-only architecture audit completed.
- [x] Human approval of architecture.
- [x] Rust project skeleton.
- [x] Mock provider and health endpoint.
- [x] First hosted provider (OpenAI-compatible).
- [x] Streaming, routing, retries, and request IDs.
- [x] Controlled CLI adapter.
- [ ] ZipChat API integration.

## Current task

Phase 4 completed: Controlled CLI adapter with strict executable allowlist, environment clearing, bounded output size, execution timeout, non-streaming and streaming support, and unit/integration tests.

## Agent update format

### 2025-05-18 (Phase 4 — Controlled CLI adapter)
- **Task**: Phase 4 implementation (`CliProvider` with executable allowlist, environment clearing, bounded buffer truncation, execution timeouts, non-streaming & streaming responses, and tests).
- **Files changed**:
  - `Cargo.toml`
  - `src/config.rs`
  - `src/providers.rs`
  - `src/main.rs`
  - `tests/api_integration_tests.rs`
  - `gateway.example.toml`
  - `PROGRESS.md`
  - `CHANGELOG.md`
  - `DECISIONS.md`
  - `BLOCKERS.md`
- **Commands run and results**:
  - `cargo check`: Passed without warnings.
  - `cargo test`: 20 passed; 0 failed.
- **Decisions or blockers**:
  - D-015 (Strict CLI sandboxing constraints) recorded in DECISIONS.md.
- **Suggested next task**:
  - Phase 5: ZipChat integration and API contract freezing.

### 2025-05-18 (Phase 3 — Streaming and routing)
- **Task**: Phase 3 implementation (SSE chat completions streaming, request ID propagation and structured logs, deterministic model routing with fallback, and bounded retries on transient errors).
- **Files changed**:
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/api.rs`
  - `src/middleware.rs`
  - `src/models.rs`
  - `src/providers.rs`
  - `src/config.rs`
  - `src/main.rs`
  - `tests/api_integration_tests.rs`
  - `PROGRESS.md`
  - `CHANGELOG.md`
  - `DECISIONS.md`
  - `BLOCKERS.md`
- **Commands run and results**:
  - `cargo check`: Passed without warnings.
  - `cargo test`: 17 passed; 0 failed.
- **Decisions or blockers**:
  - D-013 (SSE streaming format) and D-014 (bounded retries with exponential backoff) recorded in DECISIONS.md.
- **Suggested next task**:
  - Phase 4: Controlled CLI adapter design and human approval.

### 2025-05-18 (Phase 2 — First hosted provider)
- **Task**: Phase 2 implementation (`OpenAICompatibleProvider` with configurable base_url, bearer auth, secret redaction, request timeout, bounded response handling, and wiremock mock integration tests).
- **Files changed**:
  - `Cargo.toml`
  - `src/config.rs`
  - `src/providers.rs`
  - `src/main.rs`
  - `tests/api_integration_tests.rs`
  - `gateway.example.toml`
  - `PROGRESS.md`
  - `CHANGELOG.md`
  - `DECISIONS.md`
- **Commands run and results**:
  - `cargo check`: Passed without warnings.
  - `cargo test`: 14 passed; 0 failed.
- **Decisions or blockers**:
  - D-011, D-012 recorded in DECISIONS.md.
- **Suggested next task**:
  - Phase 3: Streaming (SSE) and model/provider routing enhancements.

### 2025-05-18 (Phase 1 — Skeleton and deterministic tests)
- **Task**: Phase 1 implementation (Axum + Tokio single crate, configuration loading, OpenAI-compatible error handling, mock provider, `/health`, `/v1/models`, `/v1/chat/completions`).
- **Files changed**:
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/main.rs`
  - `src/config.rs`
  - `src/error.rs`
  - `src/models.rs`
  - `src/providers.rs`
  - `src/api.rs`
  - `tests/api_integration_tests.rs`
  - `gateway.example.toml`
  - `PROGRESS.md`
  - `CHANGELOG.md`
  - `DECISIONS.md`
  - `BLOCKERS.md`
- **Commands run and results**:
  - `cargo check`: Passed (no warnings).
  - `cargo test`: 11 passed; 0 failed.
- **Decisions or blockers**:
  - D-007, D-008, D-009, D-010 recorded.
- **Suggested next task**:
  - Phase 2: First hosted provider.
