# Jules task 001 — Read-only architecture and feasibility audit

## Mode: READ ONLY

Do not create, edit, delete, or rename project source files. Do not add dependencies. Do not commit changes. Your deliverable is an audit report only (a proposed Markdown report may be returned in your response; do not write it into the repository unless the human asks).

## Context

We want a lightweight Rust AI gateway that can run on a resource-constrained Raspberry Pi without Docker and without requiring local model inference. It should unify hosted model providers, later controlled CLI integrations, and custom APIs behind a consistent API. A separate `zipchat` repository will contain the client.

LiteLLM is optional to evaluate, not a required core dependency. The user wants incremental, visible, human-approved progress—not unrestricted autonomy.

## Investigate

1. Inspect the current repository and identify what exists.
2. Recommend a minimal Rust architecture and module boundaries.
3. Compare a small number of realistic HTTP server / async runtime options.
4. Recommend configuration and secret-handling strategy.
5. Propose the smallest useful API contract for health, model listing, and chat completion.
6. Explain a sensible path to streaming.
7. Explain how to isolate provider-specific behavior behind adapters.
8. Evaluate whether LiteLLM adds value as an optional adapter and what costs it introduces.
9. Define a safe, constrained design for a future CLI adapter. Do not implement it.
10. Identify memory/CPU/network considerations for Raspberry Pi deployment.
11. Identify testing strategy, including mock provider tests.
12. List risks, unresolved questions, and deferred features.

## Deliverable

Return:
- Executive recommendation.
- Proposed directory/module tree.
- Suggested dependencies with reasons and alternatives.
- Proposed API contract (clearly label draft).
- Security and operational considerations.
- Phased implementation plan with acceptance criteria.
- Open questions requiring human decisions.
- First implementation task recommendation.

## Acceptance criteria

- No files changed and no dependencies added.
- Recommendations are specific and trade-offs are explained.
- No unsupported assumptions about the exact Raspberry Pi model.
- No arbitrary CLI execution design.
- Clearly distinguish facts, assumptions, and recommendations.
- End by asking for approval before implementation.
