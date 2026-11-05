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
