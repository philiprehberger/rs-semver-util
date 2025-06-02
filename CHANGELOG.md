# Changelog

## 0.2.1 (2026-03-22)

- Fix CHANGELOG formatting

## 0.2.0 (2026-03-21)

- Add Default trait implementation for Version (0.0.0)
- Add Display trait implementation for VersionRange
- Add FromStr trait implementation for VersionRange
- Add Version::is_pre_release() and Version::is_stable() helper methods
- Add #[must_use] attributes on version manipulation and parsing methods

## 0.1.6 (2026-03-17)

- Add readme, rust-version, documentation to Cargo.toml
- Add Development section to README

## 0.1.5 (2026-03-16)

- Update install snippet to use full version

## 0.1.4 (2026-03-16)

- Add README badges
- Synchronize version across Cargo.toml, README, and CHANGELOG

## 0.1.0 (2026-03-15)

- Initial release
- Semantic version parsing with pre-release support
- Version range parsing: caret, tilde, wildcard, comparison, compound
- Version bumping (major, minor, patch, pre-release)
- Semver-compliant ordering including pre-release
- Zero dependencies
