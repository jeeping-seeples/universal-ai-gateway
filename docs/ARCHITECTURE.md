# Proposed architecture (initial hypothesis)

This is a proposal for the audit to validate, not an approved implementation design.

```text
HTTP API
  |
  v
Request validation / API types
  |
  v
Router and policy layer
  |
  +--> Provider registry
  |      +--> Hosted provider adapter(s)
  |      +--> Mock provider (tests)
  |
  +--> Controlled CLI adapter (later phase; constrained)
  |
  v
Normalized response / streaming events
```

## Boundaries

- **API layer:** HTTP routes, request parsing, response serialization.
- **Core:** normalized request/response types, provider trait, routing decisions, error types.
- **Adapters:** provider-specific HTTP requests and response mapping.
- **Configuration:** validated settings and secret references.
- **Observability:** request IDs, structured logs, redaction.

The audit should recommend a concrete module layout and dependency set. Avoid premature abstraction and avoid a generic workflow engine in the first milestone.
