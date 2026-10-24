# Changelog

All notable changes to `phi-tools` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-10-24

### Added
- **Lazy Chrome lifecycle** — `BrowserToolset::lazy` / `BrowserSession::lazy`:
  the instance spawns on the first tool call, not at registration. A process
  that merely *has* the 22 browser tools must not drag a Chrome into every
  run. Three states on purpose: `Idle` (never paid for Chrome yet), `Running`,
  and `Down` (torn down on purpose — only `browser_restart` brings it back,
  and the error says so instead of silently respawning what the agent just
  closed).
- **`browser_tools()`** — the 22 tools as a `Vec<Arc<dyn Tool>>` so product
  shells that wrap (rather than are) `agent_base::AgentBuilder` can register
  them with their own `register_tool_arc`; `register_browser_tools` is now
  the same list folded onto a plain builder.
- **`browser_screenshot` gains `save_to`** — also write the PNG to a path for
  hand-off to something that reads files (a canvas reference image); the
  image is still returned either way, and a write failure never fails the
  capture.

### Changed
- Browser session API is tab-centric and resumable (`tab`/`get_tabs`/
  `get_active_tab`/`navigate`/`extract_dom`/… mutate through `&mut self` and
  reattach from the build config); the DOM tree is serializable with
  `find_by_index` lookup, the selector index map grows to cover arbitrary
  ARIA indices, and interactability checks require `pointer`.
- Bump `agent-base` requirement to 0.11.0 (per-call timeout budget).

## [0.6.2] - 2026-10-18

### Changed
- Bump `agent-base` requirement to 0.10.0 (tool timeout / registry fixes live
  upstream). No functional changes in this crate.

## [0.6.1] - 2026-10-07

### Changed
- **Dependency:** `agent-base` bumped from 0.8.0 to 0.9.0, keeping the family on
  a single agent-base version alongside phi-agent 0.18.0. No API changes.

## [0.6.0] - 2026-09-27

### Changed
- **Breaking (dependency):** `agent-base` bumped from 0.5.0 to 0.8.0, keeping
  the framework on a single agent-base version alongside phi-agent 0.17.0.
  No public API changes in phi-tools itself.

## [0.5.0] - 2026-09-06

### Changed
- **Breaking (dependency):** `agent-base` bumped from 0.4.0 to 0.5.0, keeping
  the framework on a single agent-base version alongside phi-agent 0.14.0.
  No public API changes in phi-tools itself.
- CI and codecov badges track the `main` branch (migrated from `master`).
