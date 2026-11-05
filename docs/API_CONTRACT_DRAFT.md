# API contract draft

This is a discussion draft only. Do not implement or treat these paths as frozen until the audit and human review approve them.

Potential first endpoints:
- `GET /health` — process health.
- `GET /v1/models` — models exposed by configured adapters.
- `POST /v1/chat/completions` — normalized chat request; initially non-streaming.
- Streaming mode may be added after non-streaming behavior and error semantics are tested.

The audit must define:
- Required and optional request fields.
- Model naming and provider selection rules.
- Error response schema and HTTP status mapping.
- Timeout behavior.
- Streaming event format and termination behavior.
- Request ID propagation.
- Authentication expectations, including safe localhost-only defaults if no auth is configured.

Do not claim compatibility with the full OpenAI API unless the supported subset is tested and documented.
