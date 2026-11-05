# Security requirements

- Treat provider API keys as secrets. Never commit them.
- Redact authorization headers, API keys, and secret-bearing configuration from logs and errors.
- Use request timeouts and bounded response sizes.
- Validate model names and user-supplied parameters.
- Avoid arbitrary shell execution.
- If a CLI adapter is approved later, constrain executable paths, arguments, environment, working directory, runtime, output size, and permissions.
- Do not expose a network listener beyond localhost by default unless an authentication design is in place.
- Document whether request content is sent to third-party providers.
