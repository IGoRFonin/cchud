# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] — 2026-04-26

### Added

- Initial repository skeleton (`Cargo.toml`, `rust-toolchain.toml`, `.gitignore`).
- Skeleton binary: reads stdin, prints `cchud (skeleton) | input bytes: N`.
- Snapshot-test harness (`tests/snapshots.rs`) over Phase 0 payload fixtures (12 samples).
- GitHub Actions CI matrix (macOS + Ubuntu + Windows): build, test, clippy, fmt; gated by actionlint.
- Hyperfine bench workflow on PR (macOS-only).
- Dependabot for cargo + github-actions, weekly cadence.
- README, LICENSE (MIT), ATTRIBUTION (credits ccstatusline upstream), this CHANGELOG.

### Notes

- No widgets yet. Real pipeline lands in Phase 2.
- Repository is private; switch to public is a manual decision after content review.

[Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
