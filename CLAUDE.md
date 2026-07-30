# CLAUDE.md — phi-tools

Agent toolset: local shell and file operations.

## Rules

### Dependencies
- `Cargo.toml` uses **pure version deps** (no `path`).
- To debug against a local dependency: temporarily add `path`, **DO NOT commit** it.

### Publishing
1. Bump version → commit → push → `cargo publish --registry crates-io`

### Downstream
- [ ] phi-agent (if the version constraint changed)
