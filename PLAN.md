# Project plan

Status: planning / audit not yet approved

## Phase 0 — Read-only feasibility audit

- Inspect repository state and identify existing code (if any).
- Recommend a minimal Rust architecture suitable for a low-memory Raspberry Pi.
- Compare candidate HTTP frameworks and async runtimes.
- Define provider adapter boundaries and configuration approach.
- Propose a minimal OpenAI-compatible API surface.
- Identify security, secret-handling, timeout, streaming, and observability requirements.
- Identify what should be deferred.
- Produce a phased implementation plan with estimates and acceptance criteria.

**Exit criteria:** human reviews and approves the audit before implementation begins.

## Phase 1 — Skeleton and deterministic tests

- Create Cargo workspace/package structure.
- Add health endpoint.
- Add configuration loading and validation.
- Add mock provider.
- Add structured error model.
- Add unit and integration tests.

## Phase 2 — First hosted provider

- Implement one OpenAI-compatible provider adapter.
- Support non-streaming chat first.
- Add safe secret configuration and redacted diagnostics.
- Add request timeout and bounded response handling.
- Add provider capability metadata.

## Phase 3 — Streaming and routing

- Add server-sent events or another justified streaming mechanism.
- Add deterministic model/provider routing.
- Add bounded retries only for safe retry cases.
- Add request IDs and useful logs without secrets.

## Phase 4 — Controlled CLI adapter

- Design a narrow adapter interface.
- Do not expose arbitrary shell execution as a public API.
- Define executable allowlists, argument validation, working-directory policy, timeouts, output limits, and approval requirements.
- Implement only after explicit human approval of the design.

## Phase 5 — ZipChat integration

- Publish and freeze the API contract.
- Add compatibility tests and an example client request.
- Coordinate with the separate ZipChat repository.

## Release readiness

- Document configuration and deployment without Docker.
- Provide a Raspberry Pi resource-conscious deployment guide.
- Document limitations and threat model.
- Verify tests on supported target platforms where possible.
