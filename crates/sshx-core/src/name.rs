//! Validated identifiers that are safe to embed in a URL path.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

/// A name that is safe to interpolate into a URL path without escaping.
///
/// Session names are generated with [`rand_alphanumeric`], so this is a
/// whitelist rather than a blacklist: it rejects anything that could change
/// the meaning of a URL path, such as `/`, `.`, or `%`. Interpolating a
/// `SafeName` into a path is therefore always correct, so callers never need
/// to escape it.
///
/// [`rand_alphanumeric`]: crate::rand_alphanumeric
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SafeName(String);

impl SafeName {
    /// Maximum accepted length, in bytes.
    ///
    /// This is far above the length of any generated name, and exists only to
    /// bound the work done on attacker-controlled input.
    pub const MAX_LEN: usize = 64;

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SafeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for SafeName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl FromStr for SafeName {
    type Err = InvalidName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(InvalidName::Empty);
        }
        if s.len() > Self::MAX_LEN {
            return Err(InvalidName::TooLong);
        }
        if !s.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err(InvalidName::NotAlphanumeric);
        }
        Ok(SafeName(s.to_string()))
    }
}

impl Serialize for SafeName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

/// Deserializes by way of [`SafeName::from_str`], so the whitelist is enforced
/// by every deserializer and not just by explicit `parse` calls.
///
/// This is what lets `axum::extract::Path<SafeName>` reject an invalid name on
/// its own: axum deserializes path parameters, so a bad name fails extraction
/// with `400 Bad Request` before the handler (and any peer proxying) runs.
impl<'de> Deserialize<'de> for SafeName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SafeNameVisitor;

        impl Visitor<'_> for SafeNameVisitor {
            type Value = SafeName;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a non-empty alphanumeric name")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<SafeName, E> {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(SafeNameVisitor)
    }
}

/// The reason a string was rejected as a [`SafeName`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidName {
    /// The name was the empty string.
    Empty,

    /// The name exceeded [`SafeName::MAX_LEN`].
    TooLong,

    /// The name contained a byte outside `A-Z`, `a-z`, and `0-9`.
    NotAlphanumeric,
}

impl fmt::Display for InvalidName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Empty => "name is empty",
            Self::TooLong => "name is too long",
            Self::NotAlphanumeric => "name must be purely alphanumeric",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for InvalidName {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_alphanumeric() {
        for name in ["aB3xY9zQ1mK", "0123456789", "a", "Z"] {
            assert_eq!(name.parse::<SafeName>().unwrap().as_str(), name);
        }
    }

    #[test]
    fn rejects_path_metacharacters() {
        // Each of these would change the meaning of an interpolated URL path.
        for name in [
            "",
            "..",
            "a/b",
            "a%2Fb",
            "../../admin",
            "a b",
            "a\tb",
            "a\nb",
            "a?b",
            "a#b",
            "a\\b",
            "ünïcode",
            "a\0b",
        ] {
            assert!(
                name.parse::<SafeName>().is_err(),
                "expected {name:?} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_overlong() {
        assert!("a".repeat(SafeName::MAX_LEN).parse::<SafeName>().is_ok());
        assert!("a"
            .repeat(SafeName::MAX_LEN + 1)
            .parse::<SafeName>()
            .is_err());
    }

    #[test]
    fn roundtrips_through_display() {
        let name: SafeName = "abcDEF1234".parse().unwrap();
        assert_eq!(name.to_string(), "abcDEF1234");
        assert_eq!(name.as_str(), "abcDEF1234");
    }

    #[test]
    fn generated_names_are_always_safe() {
        // Guards the invariant that makes interpolating a session name into a
        // URL path safe. If the generator's alphabet ever widens past
        // alphanumeric, this fails rather than silently opening a hole.
        for len in [1, 10, 14, 22] {
            for _ in 0..64 {
                let name = crate::rand_alphanumeric(len);
                assert!(
                    name.parse::<SafeName>().is_ok(),
                    "generated name {name:?} (len {len}) is not a SafeName"
                );
            }
        }
    }
}
