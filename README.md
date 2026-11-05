# Universal AI Gateway

A lightweight Rust service that provides one consistent API over hosted AI providers, controlled CLI agents, and custom integrations.

## Goals

- One stable API for model discovery, chat requests, streaming, and health/status.
- Keep provider integrations and routing in this repository.
- Support hosted APIs first; do not require local model inference.
- Run efficiently on modest hardware, including a Raspberry Pi.
- No Docker requirement for development or deployment.
- Make routing, retries, timeouts, and provider capabilities explicit and observable.
- Keep execution human-steered: plans and progress are visible, and consequential actions require approval.

## Non-goals for the first milestone

- Training or hosting models.
- Unrestricted autonomous agent execution.
- A large plugin marketplace.
- Kubernetes, distributed clustering, or complex multi-tenant infrastructure.
- Making LiteLLM a required runtime dependency.

## Initial direction

Use Rust with a small async HTTP server. Start with one OpenAI-compatible hosted provider and a mock provider for deterministic tests. Add one controlled CLI adapter only after the HTTP/provider contract is stable.

LiteLLM may be evaluated as an optional compatibility adapter, but the core gateway must not depend on it.

## Repositories

`zipchat` is a separate client application. It consumes the documented gateway API and must not duplicate provider integrations.

## Development

See [AGENTS.md](AGENTS.md), [PLAN.md](PLAN.md), and [docs/](docs/).
