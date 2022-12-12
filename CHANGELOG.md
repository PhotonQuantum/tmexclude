# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
 
[Unreleased]: https://github.com/PhotonQuantum/tmexclude/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/PhotonQuantum/tmexclude/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/PhotonQuantum/tmexclude/releases/tag/v0.2.0