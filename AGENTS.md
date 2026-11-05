# Agent instructions

## Operating mode

Work incrementally and keep the human in control. Do not assume permission for broad autonomous changes.

1. Read `README.md`, `PLAN.md`, `DECISIONS.md`, `PROGRESS.md`, and relevant docs before changing code.
2. For each task, state the goal, proposed files, approach, risks, and acceptance criteria.
3. If requirements are ambiguous or a decision affects public APIs, ask before implementation.
4. Make the smallest coherent change.
5. Run relevant formatting, linting, and tests; report exact commands and outcomes.
6. Update `PROGRESS.md` and `CHANGELOG.md` for meaningful work.
7. Stop at the task boundary and summarize changes, limitations, and the next proposed task.

## Approval boundaries

Ask before:
- Adding a required external service or runtime dependency.
- Changing public API contracts or persisted data formats.
- Adding shell execution, arbitrary code execution, or filesystem access.
- Introducing telemetry that sends data outside the machine.
- Adding secrets, credentials, or real provider account configuration.
- Large refactors or changing the project's language/framework.

Never:
- Commit secrets, tokens, private keys, or `.env` files.
- log API keys or full authorization headers.
- execute arbitrary user-provided shell commands without an explicit, constrained design and approval.
- claim tests passed unless they were actually run.

## Architecture constraints

- Rust-first implementation.
- Hosted provider calls are the initial target.
- No Docker requirement.
- No local model inference requirement.
- Keep provider-specific code behind internal traits/adapters.
- Keep HTTP transport, routing, provider adapters, and configuration separated.
- Make timeouts and error handling explicit.
- Keep resource use modest; avoid unnecessary background workers and heavyweight dependencies.

## First task

Perform the read-only architecture and feasibility audit in `docs/JULES_FIRST_TASK.md`. Do not modify source files during that task.
