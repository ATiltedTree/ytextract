use reqwest::Url;
use serde_json::Value;

/// A Thumbnail.
#[derive(Debug)]
pub struct Thumbnail {
    /// The [`Url`] where the [`Thumbnail`] can be found.
    pub url: Url,

    /// The width of the [`Thumbnail`]
    pub width: u64,

    /// The height of the [`Thumbnail`]
    pub height: u64,
}

impl From<&Value> for Thumbnail {
    fn from(v: &Value) -> Self {
        Thumbnail {
            url: v["url"]
                .as_str()
                .expect("A Thumbnail did not have a URL")
                .parse()
                .expect("A Thumbnails URL was not parsable"),
            width: v["width"]
                .as_u64()
                .expect("A Thumbnail did not have a width"),
            height: v["height"]
                .as_u64()
                .expect("A Thumbnail did not have a height"),
        }
    }
}
