# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-10-20

### Fixed

- Release workflow: pass `GITHUB_TOKEN` to `tauri-action` so the DMG uploads to the GitHub release. The v0.1.0 run built the DMG but failed to publish it.

### Added

- Voice key (13) on a session page: hold to push to talk into the session. A tap flashes HOLD on the tile.
- Push-to-talk: `push_to_talk` action, per-key `hold` action, `[input] hold_ms`, and holding a Claude session key to dictate in it
- Context-aware sort button on button 14 (#9)
- Numpad-style input number page for entering numbers directly on the device (#8)
- Context-aware search button on button 15 (#7)
- 100% test coverage enforced via `cargo llvm-cov` (#6)
- `CentyIssueActions` screen (#4)
- Persistent structured logging via `tracing` (#2)
- Persistent global device state stored in a TOML file (#1)

### Changed

- Replaced the custom spinner with `throbber-widgets-tui` (#5)
- Stricter Clippy lint baseline to keep code quality high: enabled `clippy::doc_markdown`, `clippy::panic_in_result_fn`, `clippy::manual_let_else`, and `clippy::uninlined_format_args` (#11, #14, #17, #27)

### Fixed

- Replaced all `unwrap`/`expect` calls to eliminate crash risks (#3)
