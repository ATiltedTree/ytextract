/// Errors produced by this Library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A Error that can occur when requesting a web page
    #[error("An Error occurred while requesting a page: {0}")]
    Request(#[from] reqwest::Error),

    /// A Error that occurs when a specific piece of data is missing.
    ///
    /// This is a bug.
    ///
    /// Report it here: <https://github.com/ATiltedTree/youtube-extractor/issues/new>
    #[error("Unable to find important data in the watch page. This is a bug. Please report it here: https://github.com/ATiltedTree/youtube-extractor/issues/new")]
    MissingData,

    /// A Error that occurs when JSON cannot be parsed
    #[error("JSON data was unable to be parsed: {0}")]
    JSON(#[from] serde_json::Error),

    /// A Error that occurred while parsing a [`Id`][crate::Id]
    #[error("{0}")]
    Id(#[from] crate::id::Error),
}
