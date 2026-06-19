# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
