//! Playlists

#[cfg(test)]
mod test;
pub mod video;

pub use self::video::Video;

use std::sync::Arc;

use crate::{
    youtube::{browse, innertube::Browse},
    Thumbnail,
};

/// The [`Error`](std::error::Error) produced when a invalid [`Playlist`] is
/// encountered
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`Playlist`] encountered an alert
    #[error("Playlist with id '{0}' encountered alert: '{alert}'")]
    Alert {
        /// The [`Id`] identifying the invalid [`Playlist`]
        id: Id,
        #[doc(hidden)]
        alert: browse::playlist::AlertRenderer,
    },
}

/// A [`Id`](crate::Id) describing a Playlist.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Id {
    /// A regular playlist with a `34` character [`Id`]
    Normal(crate::Id<34>),

    /// `WL`
    WatchLater,

    /// `RDMM`
    MyMix,

    /// `LL`
    LikedVideos,
}

/// The [`Error`](std::error::Error) produced when a invalid [`Id`] is
/// encountered
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    /// A invalid [`Id`] was found.
    ///
    /// A [`Id`] is only valid when all characters are on of:
    ///
    /// - `0..=9`
    /// - `a..=z`
    /// - `A..=Z`
    /// - `_`
    /// - `-`
    #[error("Found invalid id: '{0}'")]
    InvalidId(String),

    /// A [`Id`] had an invalid length. All [`Id`]s have to be 34 characters
    /// long
    #[error(transparent)]
    InvalidLength(#[from] crate::id::Error),
}

impl std::str::FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIXES: &[&str] = &["https://www.youtube.com/playlist?list="];

        let id = PREFIXES
            .iter()
            .find_map(|prefix| s.strip_prefix(prefix))
            // No Prefix matched. Possibly naked id (OLWUqW4BRl4). Length and
            // correctness will be checked later.
            .unwrap_or(s);

        if id.chars().all(crate::id::validate_char) {
            match id {
                "WL" => Ok(Self::WatchLater),
                "RDMM" => Ok(Self::MyMix),
                "LL" => Ok(Self::LikedVideos),
                _ => Ok(Self::Normal(id.parse()?)),
            }
        } else {
            Err(IdError::InvalidId(s.to_string()))
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Normal(id) => id.fmt(f),
            Id::WatchLater => f.write_str("WL"),
            Id::MyMix => f.write_str("RDMM"),
            Id::LikedVideos => f.write_str("LL"),
        }
    }
}

/// A Playlist.
pub struct Playlist {
    client: Arc<crate::Client>,
    response: browse::playlist::Root,
}

impl Playlist {
    pub(crate) async fn get(client: Arc<crate::Client>, id: Id) -> crate::Result<Self> {
        let response: browse::playlist::Root = client.api.browse(Browse::Playlist(id)).await?;
        if let Some((alert,)) = &response.alerts {
            if &alert.alert_renderer.r#type == "ERROR" {
                return Err(crate::Error::Playlist(Error::Alert {
                    id,
                    alert: alert.alert_renderer.clone(),
                }));
            }
        }

        Ok(Self { client, response })
    }

    fn microformat(&self) -> &browse::playlist::MicroformatDataRenderer {
        &self
            .response
            .microformat
            .as_ref()
            .expect("No microformat was found")
            .microformat_data_renderer
    }

    /// The title of a playlist.
    pub fn title(&self) -> &str {
        self.microformat().title.as_ref().expect("Title is missing")
    }

    /// The description of a playlist.
    pub fn description(&self) -> &str {
        self.microformat()
            .description
            .as_ref()
            .expect("Description is missing")
    }

    /// Is this playlist unlisted?
    pub fn unlisted(&self) -> bool {
        self.microformat().unlisted.expect("Unlisted is missing")
    }

    /// The [`Thumbnails`](Thumbnail) of a playlist.
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self
            .microformat()
            .thumbnail
            .as_ref()
            .expect("Thumbnails are missing")
            .thumbnails
    }

    /// The [`Videos`](Video) of a playlist.
    pub fn videos(&self) -> impl futures_core::Stream<Item = Result<Video, video::Error>> + '_ {
        async_stream::stream! {
            let mut videos: Box<dyn Iterator<Item = browse::playlist::PlaylistItem>> = Box::new(self.response.videos().cloned());

            while let Some(video) = videos.next() {
                match video {
                    browse::playlist::PlaylistItem::PlaylistVideoRenderer(video) => {
                        yield Video::new(Arc::clone(&self.client), video);
                    }
                    browse::playlist::PlaylistItem::ContinuationItemRenderer(continuation) => {
                        debug_assert!(videos.next().is_none(), "Found a continuation in the middle of videos!");
                        let response: browse::continuation::Root = self.client.api.continuation(continuation.get()).await.expect("Continuation request failed");
                        videos = Box::new(response.into_videos());
                    }
                }

            }
        }
    }
}

impl std::fmt::Debug for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Playlist")
            .field("title", &self.title())
            .field("description", &self.description())
            .field("unlisted", &self.unlisted())
            .field("thumbnails", &self.thumbnails())
            .finish()
    }
}
