# rs-semver-util

[![CI](https://github.com/philiprehberger/rs-semver-util/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-semver-util/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-semver-util.svg)](https://crates.io/crates/philiprehberger-semver-util)
[![License](https://img.shields.io/github/license/philiprehberger/rs-semver-util)](LICENSE)

Semantic versioning with range parsing, comparison, and bumping.

## Installation

```toml
[dependencies]
philiprehberger-semver-util = "0.1.5"
```

## Usage

```rust
use philiprehberger_semver_util::{Version, VersionRange};

// Parse and compare versions
let v1 = Version::parse("1.2.3").unwrap();
let v2 = Version::parse("1.3.0").unwrap();
assert!(v1 < v2);

// Bump versions
let v3 = v1.bump_minor();
assert_eq!(v3.to_string(), "1.3.0");

// Parse and match ranges
let range = VersionRange::parse("^1.2.0").unwrap();
assert!(range.matches(&v1));
assert!(range.matches(&v2));
assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
```

## API

| Function / Type | Description |
|-----------------|-------------|
| `Version::parse(s)` | Parse a semver string |
| `Version::new(maj, min, pat)` | Create a version |
| `.bump_major()` | Bump major version |
| `.bump_minor()` | Bump minor version |
| `.bump_patch()` | Bump patch version |
| `.bump_pre(label)` | Set/increment pre-release |
| `VersionRange::parse(s)` | Parse a version range |
| `.matches(version)` | Check if version satisfies range |
| `sort_versions(versions)` | Sort versions in semver order |

## License

MIT
