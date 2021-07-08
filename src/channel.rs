//! Channel types.

/// A [`Id`](crate::Id) describing a Channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(crate::Id<24>);

/// The [`Error`](std::error::Error) produced when a invalid [`Id`] is
/// encountered
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    /// A invalid [`Id`] was found.
    ///
    /// A [`Id`] is only valid when all characters are:
    ///
    /// - `0..=9`
    /// - `a..=z`
    /// - `A..=Z`
    /// - `_`
    /// - `-`
    #[error("Found invalid id: '{0}'")]
    InvalidId(String),

    /// A [`Id`] had an invalid length. All [`Id`]s have to be 24 characters
    /// long
    #[error(transparent)]
    InvalidLength(#[from] crate::id::Error),
}

impl std::str::FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIXES: [&str; 1] = ["https://www.youtube.com/channel/"];

        let id = PREFIXES
            .iter()
            .find_map(|prefix| s.strip_prefix(prefix))
            // No Prefix matched. Possibly naked id (UC_8wa1VbAAH-ksQ7aH3hkkg).
            // Length and correctness will be checked later.
            .unwrap_or(s);

        if id.chars().all(crate::id::validate_char) {
            Ok(Self(id.parse()?))
        } else {
            Err(IdError::InvalidId(s.to_string()))
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
