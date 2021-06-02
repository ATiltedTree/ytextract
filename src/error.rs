/// Errors produced by this Library
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A Error that can occur when requesting web content
    #[error("An Error occurred while requesting web content: {0}")]
    Request(#[from] reqwest::Error),

    /// A Error that occurs when querying a [`Video`](crate::Video).
    #[error(transparent)]
    Video(#[from] crate::video::Error),

    /// A Error that occurred while parsing a [`Id`](crate::Id)
    #[error(transparent)]
    Id(#[from] crate::id::Error),

    /// A Error that occurred while handling Streams
    #[error(transparent)]
    Stream(#[from] crate::stream::Error),
}
