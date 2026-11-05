# Architecture decisions

## D-001 — Gateway and client are separate repositories
The gateway owns provider integrations, routing, orchestration, and the HTTP API. ZipChat is a client and must consume that API rather than reimplement provider adapters.

## D-002 — Rust is the intended implementation language
The project is intended to be Rust-based. The feasibility audit may recommend specific libraries but must not silently change languages.

## D-003 — Raspberry Pi-friendly operation
The gateway should be efficient on modest hardware. Hosted model inference is the initial assumption; the Pi runs the gateway, not necessarily the model.

## D-004 — No Docker requirement
Development and deployment must be possible without Docker.

## D-005 — LiteLLM is optional
LiteLLM may be assessed as an optional adapter or compatibility aid. It is not a required core dependency.

## D-006 — Human-steered development and execution
Agents must expose plans and progress, make small changes, and pause at approval boundaries. Unrestricted autonomy is out of scope.

## D-007 — Framework and runtime: Axum and Tokio
Axum and Tokio provide low-overhead asynchronous HTTP handling with minimal allocations and easy composability on constrained hardware like Raspberry Pi 4B.

## D-008 — Configuration: TOML with environment variable overrides
Configuration is loaded from TOML (`gateway.toml`) with environment variable overrides to allow safe deployment and secrets management without checking in credentials.

## D-009 — Default localhost binding on port 9471
To prevent accidental exposure without authentication and avoid collision with common local services, default binding is strictly `127.0.0.1:9471`.

## D-010 — Single-crate architecture for initial phases
A single crate structure with modular library modules (`api`, `config`, `error`, `models`, `providers`) keeps build times and overhead minimal for development and Raspberry Pi compilation.

## D-011 — HTTP Client: Reqwest with Rustls
Reqwest with Rustls avoids external C library/OpenSSL dependencies on the Raspberry Pi while providing safe async HTTP/1.1 and HTTP/2 communication with hosted providers.

## D-012 — Secret protection via SecretString
API keys and tokens are wrapped in `SecretString` to prevent accidental logging or display in diagnostics and errors.

## D-013 — Streaming via standard Server-Sent Events (SSE)
Chat completion streaming uses `text/event-stream` returning standard OpenAI-compatible chunks terminated with `[DONE]`.

## D-014 — Bounded retries on transient errors with backoff
Upstream calls to providers retry only on transient status codes (429, 503, 504) or connection drops, bounded by `gateway.max_retries` with exponential backoff.
