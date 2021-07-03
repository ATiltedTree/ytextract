use reqwest::Url;

/// A Thumbnail.
#[serde_with::serde_as]
#[derive(Debug, serde::Deserialize, Clone)]
pub struct Thumbnail {
    /// The [`Url`] where the [`Thumbnail`] can be found.
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub url: Url,

    /// The width of the [`Thumbnail`]
    pub width: u64,

    /// The height of the [`Thumbnail`]
    pub height: u64,
}
