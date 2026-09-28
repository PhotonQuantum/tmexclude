# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `exclude-hidden` rule option: automatically exclude hidden (dot-prefixed) directories, so caches and state directories created by development tools no longer need manually maintained exclusion lists.
- `protects` rule option: a per-rule allowlist that vetoes exclusion of critical paths (e.g. `.ssh`, `.gnupg`) while still allowing stale exclusions on them to be cleaned.

### Fixed

- The watcher no longer spawns an unbounded blocking task per filesystem event. Events are coalesced into batches bounded by an in-flight cap and a bounded pending set, fixing CPU saturation (over 1000% on a 10-core machine) during mass deletions such as `uv cache prune`.
- When `no-include` is enabled, directories whose entries match no rule now skip per-entry xattr queries entirely.

## [0.2.2] - 2023-01-03

### Added

- NODUMP flag support. This flag is used by DUMP(8) and BorgBackup to indicate that a file should not be backed up.
  Check their documentation for more information.

### Fixed

- No message is shown to the user when trying to save an invalid config.
- The app crashes at startup if the config is invalid.

## [0.2.1] - 2022-12-12

### Added

- i18n support for zh-Hans.
- Telemetry for critical errors.

### Fixed

- Fix a bug that the app crashes on macOS version lower than 13.0.
- Sometimes apply log can be covered by the navigation bar.

## [0.2.0] - 2022-12-07

### Added

- A new GUI interface using [tauri](https://tauri.studio/) to provide a better user experience.
- Auto update support.

### Removed

- The CLI interface has been temporarily removed. It will be re-added in a future release.
- The homebrew formula is abandoned because we are now a GUI application. A new cask might be added in the future.
 
[Unreleased]: https://github.com/PhotonQuantum/tmexclude/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/PhotonQuantum/tmexclude/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/PhotonQuantum/tmexclude/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/PhotonQuantum/tmexclude/releases/tag/v0.2.0