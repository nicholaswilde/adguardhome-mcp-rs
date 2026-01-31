# Product Guidelines

## Tone and Voice
- **Professional and Precise:** All communications, including documentation, error messages, and tool descriptions, must prioritize technical accuracy and clarity. The language should be formal and objective, instilling confidence in security-conscious users.

## Documentation and Code Style
- **Self-Documenting Code:** Prioritize clear, descriptive naming for functions, variables, and types. Code structure should be intuitive, making the logic easy to follow without excessive explanation.
- **Minimalist Comments:** Use comments sparingly, focusing on the "why" behind complex or non-obvious decisions rather than the "what". Ensure comments add significant value and are kept up-to-date.

## Error Handling and Logging
- **Explicit Error Types:** Use custom error types (e.g., leveraging `thiserror` or `anyhow`) to provide granular and meaningful error reporting throughout the application.
- **Structured Logging:** Implement structured logging (e.g., using the `tracing` crate) to ensure that logs are both human-readable and easily parsed by automated tools for diagnostics and monitoring.

## MCP Tool Design
- **Granular and Atomic Tools:** Each MCP tool must have a single, well-defined purpose. For example, separate tools should be used for retrieving status and updating configuration. This atomicity ensures that AI models can reliably predict and compose tool calls to achieve complex goals.
