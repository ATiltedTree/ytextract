//! Error produced by YouTube and this library

/// Errors produced by this Library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A Error that can occur when requesting web content
    #[error("An Error occurred while requesting web content: {0}")]
    Request(#[from] ureq::Error),

    /// A Error reported by YouTube
    #[error(transparent)]
    Youtube(#[from] Youtube),
}

/// A Error reported by YouTube.
///
/// This Error is `#[non_exhaustive]` because YouTube can add errors at any
/// moment and breaking major version every time is annoying.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
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

    /// A [`Video`](crate::Video) is age-restricted and its
    /// [`Streams`](crate::Stream) could not be fetched.
    #[error("a video is age-restricted")]
    AgeRestricted,

    /// A [`Video`](crate::Video) is not available due to nudity or sexual content violations.
    #[error("a video is not available due to nudity or sexual content violations")]
    NudityOrSexualContentViolation,

    /// The channel or the channel associated with a video was terminated.
    #[error("the channel or the channel associated with a video was terminated")]
    AccountTerminated,

    /// The channel or the channel associated with a video was deleted.
    #[error("the channel or the channel associated with a video was deleted")]
    AccountDeleted,

    /// A [`Video`](crate::Video) was removed by the uploader.
    #[error("a video was removed by the uploader")]
    RemovedByUploader,

    /// A [`Video`](crate::Video) is not available due to violations of YouTube's Terms of Service.
    #[error("a video is not available due to violations of YouTube's Terms of Service")]
    TermsOfServiceViolation,

    /// A [`Video`](crate::Video) is not available due to a privacy claim by a third party`
    #[error("a video is not available due to a privacy claim by a third party")]
    PrivacyClaim,

    /// A [`Video`](crate::Video) is not available due to a copyright claim by the `claiment`
    #[error("a video is not available due to a copyright claim by '{claiment}'")]
    CopyrightClaim {
        /// The person that made this copyright claim
        claiment: String,
    },
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
