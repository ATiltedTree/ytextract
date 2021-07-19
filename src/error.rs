//! Error produced by YouTube and this library

/// Errors produced by this Library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A Error that can occur when requesting web content
    #[error("An Error occurred while requesting web content: {0}")]
    Request(#[from] reqwest::Error),

    /// A Error reported by YouTube
    #[error(transparent)]
    Youtube(#[from] Youtube),
}

/// A Error reported by YouTube.
///
/// This Error is `#[non_exhaustive]` because YouTube can add errors at any
/// moment and breaking major version every time is annoying.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Youtube {
    /// A entity was not found. This can be:
    ///
    /// - [`Videos`](crate::Video)
    /// - [`Playlists`](crate::Playlist)
    /// - [`Channels`](crate::Channel)
    /// - [`Streams`](crate::Stream)
    #[error("a entity was not found")]
    NotFound,

    /// A entity is private. This can be:
    ///
    /// - [`Videos`](crate::Video)
    /// - [`Playlists`](crate::Playlist)
    /// - [`Channels`](crate::Channel)
    /// - [`Streams`](crate::Stream)
    #[error("a entity is private")]
    Private,

    /// A Video is not available due to Community Guideline violations.
    #[error("a video is not available due to community guideline violations")]
    CommunityGuidelineViolation,

    /// A Video is not available in your country.
    ///
    /// Maybe try tor :^)
    #[error("a video is not available in your country")]
    GeoRestricted,

    /// A [`Stream`](crate::Stream) is only available after a purchase.
    #[error("a purchase is required to watch get this stream")]
    PurchaseRequired,

    /// A [`Playlist`](crate::Playlist) could not be viewed. Reasons being:
    ///
    /// - The playlist is a "Mix" or "My Mix" playlist
    #[error("a playlist could not be viewed")]
    Unviewable,
}

/// The Error produced when a invalid Id is found
#[derive(thiserror::Error, Debug, Clone)]
pub enum Id<const N: usize> {
    /// A invalid Id was found.
    ///
    /// A Id is only valid when all characters are on of:
    ///
    /// - `0..=9`
    /// - `a..=z`
    /// - `A..=Z`
    /// - `_`
    /// - `-`
    #[error("Found invalid id: '{0}'")]
    InvalidId(String),

    /// A Id was not the expected length
    #[error("Expected a id of length {N} but found a id of length {0}")]
    InvalidLength(usize),
}
