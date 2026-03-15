//! Semantic versioning with range parsing, comparison, and bumping.
//!
//! A zero-dependency library for working with [semver](https://semver.org/) versions
//! and version ranges.
//!
//! # Examples
//!
//! ```
//! use philiprehberger_semver_util::{Version, VersionRange};
//!
//! let v = Version::parse("1.2.3-alpha.1").unwrap();
//! assert_eq!(v.major, 1);
//! assert_eq!(v.to_string(), "1.2.3-alpha.1");
//!
//! let range = VersionRange::parse("^1.2.0").unwrap();
//! assert!(range.matches(&v));
//! ```

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A pre-release identifier, either numeric or alphanumeric.
#[derive(Clone, Debug)]
pub enum PreRelease {
    /// A numeric pre-release identifier (e.g., `1` in `alpha.1`).
    Numeric(u64),
    /// An alphanumeric pre-release identifier (e.g., `alpha` in `alpha.1`).
    AlphaNumeric(String),
}

impl PartialEq for PreRelease {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PreRelease::Numeric(a), PreRelease::Numeric(b)) => a == b,
            (PreRelease::AlphaNumeric(a), PreRelease::AlphaNumeric(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for PreRelease {}

impl Hash for PreRelease {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PreRelease::Numeric(n) => {
                state.write_u8(0);
                n.hash(state);
            }
            PreRelease::AlphaNumeric(s) => {
                state.write_u8(1);
                s.hash(state);
            }
        }
    }
}

impl Ord for PreRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PreRelease::Numeric(a), PreRelease::Numeric(b)) => a.cmp(b),
            (PreRelease::AlphaNumeric(a), PreRelease::AlphaNumeric(b)) => a.cmp(b),
            (PreRelease::Numeric(_), PreRelease::AlphaNumeric(_)) => Ordering::Less,
            (PreRelease::AlphaNumeric(_), PreRelease::Numeric(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for PreRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreRelease::Numeric(n) => write!(f, "{}", n),
            PreRelease::AlphaNumeric(s) => write!(f, "{}", s),
        }
    }
}

/// An error that occurred while parsing a version or version range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The input string was empty.
    EmptyInput,
    /// The input string had an invalid format.
    InvalidFormat(String),
    /// A numeric component could not be parsed.
    InvalidNumber(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "empty input"),
            ParseError::InvalidFormat(msg) => write!(f, "invalid format: {}", msg),
            ParseError::InvalidNumber(msg) => write!(f, "invalid number: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// A semantic version with optional pre-release identifiers.
///
/// Versions are compared according to the [semver.org](https://semver.org/) specification:
/// major, then minor, then patch, then pre-release. A version without pre-release
/// has higher precedence than one with pre-release when all other components are equal.
///
/// # Examples
///
/// ```
/// use philiprehberger_semver_util::Version;
///
/// let v = Version::parse("1.2.3").unwrap();
/// assert_eq!(v.bump_minor().to_string(), "1.3.0");
/// ```
#[derive(Clone, Debug)]
pub struct Version {
    /// The major version number.
    pub major: u64,
    /// The minor version number.
    pub minor: u64,
    /// The patch version number.
    pub patch: u64,
    /// Pre-release identifiers.
    pub pre: Vec<PreRelease>,
}

impl Version {
    /// Creates a new version with the given major, minor, and patch numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::new(1, 2, 3);
    /// assert_eq!(v.to_string(), "1.2.3");
    /// ```
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Vec::new(),
        }
    }

    /// Parses a version string.
    ///
    /// Supports versions with and without pre-release identifiers:
    /// `"1.2.3"`, `"1.2.3-alpha.1"`, `"1.2.3-beta"`.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the input is empty or malformed.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::parse("1.2.3-alpha.1").unwrap();
    /// assert_eq!(v.major, 1);
    /// assert_eq!(v.minor, 2);
    /// assert_eq!(v.patch, 3);
    /// assert_eq!(v.to_string(), "1.2.3-alpha.1");
    /// ```
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let (version_part, pre_part) = match input.find('-') {
            Some(idx) => (&input[..idx], Some(&input[idx + 1..])),
            None => (input, None),
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat(format!(
                "expected 3 version components, got {}",
                parts.len()
            )));
        }

        let major = parse_u64(parts[0])?;
        let minor = parse_u64(parts[1])?;
        let patch = parse_u64(parts[2])?;

        let pre = match pre_part {
            Some(pre_str) => parse_pre_release(pre_str)?,
            None => Vec::new(),
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    /// Returns a new version with the major number incremented.
    ///
    /// Minor and patch are reset to 0, and pre-release is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::parse("1.2.3-alpha.1").unwrap();
    /// assert_eq!(v.bump_major().to_string(), "2.0.0");
    /// ```
    pub fn bump_major(&self) -> Version {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            pre: Vec::new(),
        }
    }

    /// Returns a new version with the minor number incremented.
    ///
    /// Patch is reset to 0, and pre-release is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::parse("1.2.3").unwrap();
    /// assert_eq!(v.bump_minor().to_string(), "1.3.0");
    /// ```
    pub fn bump_minor(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            pre: Vec::new(),
        }
    }

    /// Returns a new version with the patch number incremented.
    ///
    /// Pre-release is cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::parse("1.2.3").unwrap();
    /// assert_eq!(v.bump_patch().to_string(), "1.2.4");
    /// ```
    pub fn bump_patch(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            pre: Vec::new(),
        }
    }

    /// Sets or increments a pre-release label.
    ///
    /// If the version has no pre-release, appends `label.0`. If it already has a
    /// pre-release starting with the same label and ending with a numeric identifier,
    /// that number is incremented. Otherwise, the pre-release is replaced with `label.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::Version;
    ///
    /// let v = Version::parse("1.0.0").unwrap();
    /// assert_eq!(v.bump_pre("alpha").to_string(), "1.0.0-alpha.0");
    ///
    /// let v2 = Version::parse("1.0.0-alpha.0").unwrap();
    /// assert_eq!(v2.bump_pre("alpha").to_string(), "1.0.0-alpha.1");
    ///
    /// let v3 = Version::parse("1.0.0-alpha.2").unwrap();
    /// assert_eq!(v3.bump_pre("beta").to_string(), "1.0.0-beta.0");
    /// ```
    pub fn bump_pre(&self, label: &str) -> Version {
        let new_pre = if self.pre.len() >= 2 {
            if let PreRelease::AlphaNumeric(ref existing_label) = self.pre[0] {
                if existing_label == label {
                    if let PreRelease::Numeric(n) = self.pre[self.pre.len() - 1] {
                        vec![
                            PreRelease::AlphaNumeric(label.to_string()),
                            PreRelease::Numeric(n + 1),
                        ]
                    } else {
                        vec![
                            PreRelease::AlphaNumeric(label.to_string()),
                            PreRelease::Numeric(0),
                        ]
                    }
                } else {
                    vec![
                        PreRelease::AlphaNumeric(label.to_string()),
                        PreRelease::Numeric(0),
                    ]
                }
            } else {
                vec![
                    PreRelease::AlphaNumeric(label.to_string()),
                    PreRelease::Numeric(0),
                ]
            }
        } else {
            vec![
                PreRelease::AlphaNumeric(label.to_string()),
                PreRelease::Numeric(0),
            ]
        };

        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            pre: new_pre,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-")?;
            for (i, p) in self.pre.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{}", p)?;
            }
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl Hash for Version {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
        self.pre.hash(state);
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Per semver spec: no pre-release > with pre-release
        match (self.pre.is_empty(), other.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                let len = self.pre.len().min(other.pre.len());
                for i in 0..len {
                    match self.pre[i].cmp(&other.pre[i]) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                self.pre.len().cmp(&other.pre.len())
            }
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A comparator operator used in version range expressions.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Comparator {
    Exact(Version),
    Gt(Version),
    Gte(Version),
    Lt(Version),
    Lte(Version),
}

impl Comparator {
    fn matches(&self, version: &Version) -> bool {
        match self {
            Comparator::Exact(v) => version == v,
            Comparator::Gt(v) => version > v,
            Comparator::Gte(v) => version >= v,
            Comparator::Lt(v) => version < v,
            Comparator::Lte(v) => version <= v,
        }
    }
}

/// A version range that can match against versions.
///
/// Supports caret (`^`), tilde (`~`), wildcard (`*`, `x`), comparison
/// (`>=`, `<=`, `>`, `<`), exact, and compound (comma-separated AND) ranges.
///
/// # Examples
///
/// ```
/// use philiprehberger_semver_util::{Version, VersionRange};
///
/// let range = VersionRange::parse(">=1.0.0, <2.0.0").unwrap();
/// assert!(range.matches(&Version::parse("1.5.0").unwrap()));
/// assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
///
/// let caret = VersionRange::parse("^1.2.3").unwrap();
/// assert!(caret.matches(&Version::parse("1.9.9").unwrap()));
/// assert!(!caret.matches(&Version::parse("2.0.0").unwrap()));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct VersionRange {
    comparators: Vec<Comparator>,
}

impl VersionRange {
    /// Parses a version range string.
    ///
    /// # Supported syntax
    ///
    /// - Exact: `1.2.3`
    /// - Comparison: `>=1.0.0`, `<2.0.0`, `>1.0.0`, `<=1.0.0`
    /// - Caret: `^1.2.3` (compatible with)
    /// - Tilde: `~1.2.3` (approximately)
    /// - Wildcard: `1.2.*`, `1.2.x`, `1.*`
    /// - Compound: `>=1.0.0, <2.0.0` (AND logic)
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if the input is empty or malformed.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        let mut comparators = Vec::new();

        for part in parts {
            if part.is_empty() {
                return Err(ParseError::InvalidFormat("empty range component".to_string()));
            }
            let mut comps = parse_range_part(part)?;
            comparators.append(&mut comps);
        }

        Ok(Self { comparators })
    }

    /// Returns `true` if the given version satisfies all comparators in this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_semver_util::{Version, VersionRange};
    ///
    /// let range = VersionRange::parse("^0.2.3").unwrap();
    /// assert!(range.matches(&Version::parse("0.2.5").unwrap()));
    /// assert!(!range.matches(&Version::parse("0.3.0").unwrap()));
    /// ```
    pub fn matches(&self, version: &Version) -> bool {
        self.comparators.iter().all(|c| c.matches(version))
    }
}

/// Sorts a slice of versions in semver order (ascending).
///
/// # Examples
///
/// ```
/// use philiprehberger_semver_util::{Version, sort_versions};
///
/// let mut versions = vec![
///     Version::parse("2.0.0").unwrap(),
///     Version::parse("1.0.0").unwrap(),
///     Version::parse("1.1.0").unwrap(),
/// ];
/// sort_versions(&mut versions);
/// assert_eq!(versions[0].to_string(), "1.0.0");
/// assert_eq!(versions[2].to_string(), "2.0.0");
/// ```
pub fn sort_versions(versions: &mut [Version]) {
    versions.sort();
}

// --- Internal helpers ---

fn parse_u64(s: &str) -> Result<u64, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::InvalidNumber("empty string".to_string()));
    }
    s.parse::<u64>()
        .map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

fn parse_pre_release(s: &str) -> Result<Vec<PreRelease>, ParseError> {
    if s.is_empty() {
        return Err(ParseError::InvalidFormat("empty pre-release".to_string()));
    }
    let mut result = Vec::new();
    for part in s.split('.') {
        if part.is_empty() {
            return Err(ParseError::InvalidFormat(
                "empty pre-release identifier".to_string(),
            ));
        }
        if let Ok(n) = part.parse::<u64>() {
            result.push(PreRelease::Numeric(n));
        } else {
            result.push(PreRelease::AlphaNumeric(part.to_string()));
        }
    }
    Ok(result)
}

/// Parse a version string that may have only 2 components (e.g., "1.2").
fn parse_partial_version(s: &str) -> Result<(u64, u64, Option<u64>), ParseError> {
    let parts: Vec<&str> = s.split('.').collect();
    match parts.len() {
        2 => {
            let major = parse_u64(parts[0])?;
            let minor = parse_u64(parts[1])?;
            Ok((major, minor, None))
        }
        3 => {
            let major = parse_u64(parts[0])?;
            let minor = parse_u64(parts[1])?;
            let patch = parse_u64(parts[2])?;
            Ok((major, minor, Some(patch)))
        }
        _ => Err(ParseError::InvalidFormat(format!(
            "expected 2 or 3 version components, got {}",
            parts.len()
        ))),
    }
}

fn parse_range_part(part: &str) -> Result<Vec<Comparator>, ParseError> {
    let part = part.trim();

    // Caret range
    if let Some(rest) = part.strip_prefix('^') {
        return parse_caret(rest);
    }

    // Tilde range
    if let Some(rest) = part.strip_prefix('~') {
        return parse_tilde(rest);
    }

    // Comparison operators
    if let Some(rest) = part.strip_prefix(">=") {
        let v = Version::parse(rest.trim())?;
        return Ok(vec![Comparator::Gte(v)]);
    }
    if let Some(rest) = part.strip_prefix("<=") {
        let v = Version::parse(rest.trim())?;
        return Ok(vec![Comparator::Lte(v)]);
    }
    if let Some(rest) = part.strip_prefix('>') {
        let v = Version::parse(rest.trim())?;
        return Ok(vec![Comparator::Gt(v)]);
    }
    if let Some(rest) = part.strip_prefix('<') {
        let v = Version::parse(rest.trim())?;
        return Ok(vec![Comparator::Lt(v)]);
    }

    // Wildcard / x-range
    if part.contains('*') || part.contains('x') {
        return parse_wildcard(part);
    }

    // Exact version
    let v = Version::parse(part)?;
    Ok(vec![Comparator::Exact(v)])
}

fn parse_caret(s: &str) -> Result<Vec<Comparator>, ParseError> {
    let v = Version::parse(s.trim())?;

    let upper = if v.major != 0 {
        Version::new(v.major + 1, 0, 0)
    } else if v.minor != 0 {
        Version::new(0, v.minor + 1, 0)
    } else {
        Version::new(0, 0, v.patch + 1)
    };

    Ok(vec![
        Comparator::Gte(v),
        Comparator::Lt(upper),
    ])
}

fn parse_tilde(s: &str) -> Result<Vec<Comparator>, ParseError> {
    let s = s.trim();

    let (major, minor, patch) = parse_partial_version(s)?;

    let lower = Version::new(major, minor, patch.unwrap_or(0));
    let upper = Version::new(major, minor + 1, 0);

    Ok(vec![
        Comparator::Gte(lower),
        Comparator::Lt(upper),
    ])
}

fn parse_wildcard(s: &str) -> Result<Vec<Comparator>, ParseError> {
    let parts: Vec<&str> = s.split('.').collect();

    match parts.len() {
        2 => {
            // "1.*" or "1.x"
            let major = parse_u64(parts[0])?;
            Ok(vec![
                Comparator::Gte(Version::new(major, 0, 0)),
                Comparator::Lt(Version::new(major + 1, 0, 0)),
            ])
        }
        3 => {
            // "1.2.*" or "1.2.x"
            let major = parse_u64(parts[0])?;
            let minor = parse_u64(parts[1])?;
            Ok(vec![
                Comparator::Gte(Version::new(major, minor, 0)),
                Comparator::Lt(Version::new(major, minor + 1, 0)),
            ])
        }
        _ => Err(ParseError::InvalidFormat(format!(
            "invalid wildcard range: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Version parsing ---

    #[test]
    fn parse_simple_version() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_empty());
    }

    #[test]
    fn parse_version_with_pre_release() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre.len(), 2);
        assert_eq!(v.pre[0], PreRelease::AlphaNumeric("alpha".to_string()));
        assert_eq!(v.pre[1], PreRelease::Numeric(1));
    }

    #[test]
    fn parse_version_with_single_pre_release() {
        let v = Version::parse("1.0.0-beta").unwrap();
        assert_eq!(v.pre.len(), 1);
        assert_eq!(v.pre[0], PreRelease::AlphaNumeric("beta".to_string()));
    }

    #[test]
    fn parse_version_zero() {
        let v = Version::parse("0.0.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn parse_version_large_numbers() {
        let v = Version::parse("100.200.300").unwrap();
        assert_eq!(v.major, 100);
        assert_eq!(v.minor, 200);
        assert_eq!(v.patch, 300);
    }

    #[test]
    fn parse_version_empty_input() {
        assert_eq!(Version::parse(""), Err(ParseError::EmptyInput));
    }

    #[test]
    fn parse_version_too_few_parts() {
        assert!(Version::parse("1.2").is_err());
    }

    #[test]
    fn parse_version_too_many_parts() {
        assert!(Version::parse("1.2.3.4").is_err());
    }

    #[test]
    fn parse_version_non_numeric() {
        assert!(Version::parse("a.b.c").is_err());
    }

    #[test]
    fn version_from_str() {
        let v: Version = "2.0.0".parse().unwrap();
        assert_eq!(v.major, 2);
    }

    // --- Version display ---

    #[test]
    fn display_simple_version() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn display_version_with_pre_release() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.1");
    }

    // --- Version ordering ---

    #[test]
    fn ordering_major() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_minor() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_patch() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 0, 1);
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_no_pre_greater_than_pre() {
        let v1 = Version::parse("1.0.0-alpha").unwrap();
        let v2 = Version::new(1, 0, 0);
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_pre_release_numeric() {
        let v1 = Version::parse("1.0.0-alpha.1").unwrap();
        let v2 = Version::parse("1.0.0-alpha.2").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_pre_release_alpha_vs_numeric() {
        // Numeric identifiers have lower precedence than alphanumeric
        let v1 = Version::parse("1.0.0-1").unwrap();
        let v2 = Version::parse("1.0.0-alpha").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_pre_release_alphabetical() {
        let v1 = Version::parse("1.0.0-alpha").unwrap();
        let v2 = Version::parse("1.0.0-beta").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_pre_release_longer_is_greater() {
        let v1 = Version::parse("1.0.0-alpha").unwrap();
        let v2 = Version::parse("1.0.0-alpha.1").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn ordering_equal() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 0, 0);
        assert_eq!(v1, v2);
    }

    #[test]
    fn ordering_semver_spec_example() {
        // 1.0.0-alpha < 1.0.0-alpha.1 < 1.0.0-alpha.beta < 1.0.0-beta
        // < 1.0.0-beta.2 < 1.0.0-beta.11 < 1.0.0-rc.1 < 1.0.0
        let versions = vec![
            Version::parse("1.0.0").unwrap(),
            Version::parse("1.0.0-rc.1").unwrap(),
            Version::parse("1.0.0-beta.11").unwrap(),
            Version::parse("1.0.0-beta.2").unwrap(),
            Version::parse("1.0.0-beta").unwrap(),
            Version::parse("1.0.0-alpha.beta").unwrap(),
            Version::parse("1.0.0-alpha.1").unwrap(),
            Version::parse("1.0.0-alpha").unwrap(),
        ];

        let mut sorted = versions.clone();
        sorted.sort();

        assert_eq!(sorted[0].to_string(), "1.0.0-alpha");
        assert_eq!(sorted[1].to_string(), "1.0.0-alpha.1");
        assert_eq!(sorted[2].to_string(), "1.0.0-alpha.beta");
        assert_eq!(sorted[3].to_string(), "1.0.0-beta");
        assert_eq!(sorted[4].to_string(), "1.0.0-beta.2");
        assert_eq!(sorted[5].to_string(), "1.0.0-beta.11");
        assert_eq!(sorted[6].to_string(), "1.0.0-rc.1");
        assert_eq!(sorted[7].to_string(), "1.0.0");
    }

    // --- Bump operations ---

    #[test]
    fn bump_major() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = v.bump_major();
        assert_eq!(bumped.to_string(), "2.0.0");
    }

    #[test]
    fn bump_major_clears_pre() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        let bumped = v.bump_major();
        assert_eq!(bumped.to_string(), "2.0.0");
        assert!(bumped.pre.is_empty());
    }

    #[test]
    fn bump_minor() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = v.bump_minor();
        assert_eq!(bumped.to_string(), "1.3.0");
    }

    #[test]
    fn bump_minor_clears_pre() {
        let v = Version::parse("1.2.3-beta").unwrap();
        let bumped = v.bump_minor();
        assert_eq!(bumped.to_string(), "1.3.0");
    }

    #[test]
    fn bump_patch() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = v.bump_patch();
        assert_eq!(bumped.to_string(), "1.2.4");
    }

    #[test]
    fn bump_patch_clears_pre() {
        let v = Version::parse("1.2.3-rc.1").unwrap();
        let bumped = v.bump_patch();
        assert_eq!(bumped.to_string(), "1.2.4");
    }

    #[test]
    fn bump_pre_new() {
        let v = Version::parse("1.0.0").unwrap();
        let bumped = v.bump_pre("alpha");
        assert_eq!(bumped.to_string(), "1.0.0-alpha.0");
    }

    #[test]
    fn bump_pre_increment() {
        let v = Version::parse("1.0.0-alpha.0").unwrap();
        let bumped = v.bump_pre("alpha");
        assert_eq!(bumped.to_string(), "1.0.0-alpha.1");
    }

    #[test]
    fn bump_pre_change_label() {
        let v = Version::parse("1.0.0-alpha.2").unwrap();
        let bumped = v.bump_pre("beta");
        assert_eq!(bumped.to_string(), "1.0.0-beta.0");
    }

    // --- Range parsing and matching ---

    #[test]
    fn range_exact() {
        let range = VersionRange::parse("1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn range_gte() {
        let range = VersionRange::parse(">=1.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn range_gt() {
        let range = VersionRange::parse(">1.0.0").unwrap();
        assert!(!range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.0.1").unwrap()));
    }

    #[test]
    fn range_lt() {
        let range = VersionRange::parse("<2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn range_lte() {
        let range = VersionRange::parse("<=2.0.0").unwrap();
        assert!(range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn range_caret_major() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn range_caret_zero_minor() {
        let range = VersionRange::parse("^0.2.3").unwrap();
        assert!(range.matches(&Version::parse("0.2.3").unwrap()));
        assert!(range.matches(&Version::parse("0.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("0.3.0").unwrap()));
    }

    #[test]
    fn range_caret_zero_zero() {
        let range = VersionRange::parse("^0.0.3").unwrap();
        assert!(range.matches(&Version::parse("0.0.3").unwrap()));
        assert!(!range.matches(&Version::parse("0.0.4").unwrap()));
        assert!(!range.matches(&Version::parse("0.0.2").unwrap()));
    }

    #[test]
    fn range_tilde_full() {
        let range = VersionRange::parse("~1.2.3").unwrap();
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn range_tilde_partial() {
        let range = VersionRange::parse("~1.2").unwrap();
        assert!(range.matches(&Version::parse("1.2.0").unwrap()));
        assert!(range.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn range_wildcard_star() {
        let range = VersionRange::parse("1.2.*").unwrap();
        assert!(range.matches(&Version::parse("1.2.0").unwrap()));
        assert!(range.matches(&Version::parse("1.2.99").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn range_wildcard_x() {
        let range = VersionRange::parse("1.2.x").unwrap();
        assert!(range.matches(&Version::parse("1.2.0").unwrap()));
        assert!(range.matches(&Version::parse("1.2.5").unwrap()));
        assert!(!range.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn range_wildcard_major() {
        let range = VersionRange::parse("1.*").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.99.99").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn range_compound() {
        let range = VersionRange::parse(">=1.0.0, <2.0.0").unwrap();
        assert!(range.matches(&Version::parse("1.0.0").unwrap()));
        assert!(range.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.9.9").unwrap()));
        assert!(!range.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn range_compound_tight() {
        let range = VersionRange::parse(">=1.2.3, <=1.2.5").unwrap();
        assert!(!range.matches(&Version::parse("1.2.2").unwrap()));
        assert!(range.matches(&Version::parse("1.2.3").unwrap()));
        assert!(range.matches(&Version::parse("1.2.4").unwrap()));
        assert!(range.matches(&Version::parse("1.2.5").unwrap()));
        assert!(!range.matches(&Version::parse("1.2.6").unwrap()));
    }

    // --- Range parse errors ---

    #[test]
    fn range_parse_empty() {
        assert_eq!(VersionRange::parse(""), Err(ParseError::EmptyInput));
    }

    #[test]
    fn range_parse_invalid() {
        assert!(VersionRange::parse("not-a-version").is_err());
    }

    // --- sort_versions ---

    #[test]
    fn sort_versions_basic() {
        let mut versions = vec![
            Version::parse("2.0.0").unwrap(),
            Version::parse("1.0.0").unwrap(),
            Version::parse("1.1.0").unwrap(),
            Version::parse("0.1.0").unwrap(),
        ];
        sort_versions(&mut versions);
        assert_eq!(versions[0].to_string(), "0.1.0");
        assert_eq!(versions[1].to_string(), "1.0.0");
        assert_eq!(versions[2].to_string(), "1.1.0");
        assert_eq!(versions[3].to_string(), "2.0.0");
    }

    #[test]
    fn sort_versions_with_pre_release() {
        let mut versions = vec![
            Version::parse("1.0.0").unwrap(),
            Version::parse("1.0.0-beta").unwrap(),
            Version::parse("1.0.0-alpha").unwrap(),
        ];
        sort_versions(&mut versions);
        assert_eq!(versions[0].to_string(), "1.0.0-alpha");
        assert_eq!(versions[1].to_string(), "1.0.0-beta");
        assert_eq!(versions[2].to_string(), "1.0.0");
    }

    // --- Edge cases ---

    #[test]
    fn version_equality_with_pre() {
        let v1 = Version::parse("1.0.0-alpha.1").unwrap();
        let v2 = Version::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn caret_zero_zero_zero() {
        let range = VersionRange::parse("^0.0.0").unwrap();
        assert!(range.matches(&Version::parse("0.0.0").unwrap()));
        assert!(!range.matches(&Version::parse("0.0.1").unwrap()));
    }

    #[test]
    fn version_new_has_no_pre() {
        let v = Version::new(1, 2, 3);
        assert!(v.pre.is_empty());
    }

    #[test]
    fn whitespace_trimmed() {
        let v = Version::parse("  1.2.3  ").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn range_whitespace_in_compound() {
        let range = VersionRange::parse("  >=1.0.0 ,  <2.0.0  ").unwrap();
        assert!(range.matches(&Version::parse("1.5.0").unwrap()));
    }
}
