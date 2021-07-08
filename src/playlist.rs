//! Playlists

pub mod video;

pub use self::video::Video;

use std::sync::Arc;

use crate::{
    youtube::{
        browse::{
            self,
            playlist::{
                PlaylistSidebarItem, PlaylistSidebarPrimaryInfoRenderer,
                PlaylistSidebarSecondaryInfoRenderer,
            },
        },
        innertube::Browse,
    },
    Thumbnail,
};

/// The [`Error`](std::error::Error) produced when a invalid [`Playlist`] is
/// encountered
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`Playlist`] encountered an alert
    #[error("Playlist with id '{0}' encountered alert: '{alert}'")]
    Alert {
        #[doc(hidden)]
        alert: browse::playlist::AlertRenderer,
    },
}

/// A [`Id`](crate::Id) describing a Playlist.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Id(String);

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
        const ID_PREFIXES: &[&str] = &["PL", "RD", "UL", "UU", "PU", "OL", "LL", "FL", "WL"];

        let id = PREFIXES
            .iter()
            .find_map(|prefix| s.strip_prefix(prefix))
            // No Prefix matched. Possibly naked id (OLWUqW4BRl4). Length and
            // correctness will be checked later.
            .unwrap_or(s);

        if id.chars().all(crate::id::validate_char)
            && ID_PREFIXES.iter().any(|prefix| id.starts_with(prefix))
        {
            Ok(Id(s.to_string()))
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

    fn primary_sidebar(&self) -> &PlaylistSidebarPrimaryInfoRenderer {
        self.response
            .sidebar
            .as_ref()
            .expect("No sidebar")
            .playlist_sidebar_renderer
            .items
            .iter()
            .find_map(|x| {
                if let PlaylistSidebarItem::PlaylistSidebarPrimaryInfoRenderer(x) = x {
                    Some(x)
                } else {
                    None
                }
            })
            .as_ref()
            .expect("No Primary sidebar")
    }

    fn secondary_sidebar(&self) -> Option<&PlaylistSidebarSecondaryInfoRenderer> {
        self.response
            .sidebar
            .as_ref()
            .expect("No sidebar")
            .playlist_sidebar_renderer
            .items
            .iter()
            .find_map(|x| {
                if let PlaylistSidebarItem::PlaylistSidebarSecondaryInfoRenderer(x) = x {
                    Some(x)
                } else {
                    None
                }
            })
    }

    /// The [`Id`] of a playlist
    pub fn id(&self) -> Id {
        self.response
            .contents
            .as_ref()
            .expect("No content")
            .two_column_browse_results_renderer
            .tabs
            .0
            .tab_renderer
            .content
            .section_list_renderer
            .contents
            .0
            .item_section_renderer
            .contents
            .0
            .playlist_video_list_renderer
            .playlist_id
            .clone()
    }

    /// The title of a playlist.
    pub fn title(&self) -> &str {
        &self.microformat().title
    }

    /// The description of a playlist.
    pub fn description(&self) -> &str {
        &self.microformat().description
    }

    /// The name of the author of this playlist
    pub fn author(&self) -> Option<&str> {
        self.secondary_sidebar()
            .as_ref()
            .map(|x| x.video_owner.video_owner_renderer.name())
    }

    /// The id of the author's channel of this playlist
    pub fn channel_id(&self) -> Option<crate::channel::Id> {
        self.secondary_sidebar()
            .as_ref()
            .map(|x| x.video_owner.video_owner_renderer.id())
    }

    /// Is this playlist unlisted?
    pub fn unlisted(&self) -> bool {
        self.microformat().unlisted
    }

    /// The [`Thumbnails`](Thumbnail) of a playlist.
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self.microformat().thumbnail.thumbnails
    }

    /// The amount of views of a playlist
    pub fn views(&self) -> u64 {
        self.primary_sidebar().stats.1.as_number()
    }

    /// The amount of videos in a playlist
    pub fn length(&self) -> u64 {
        self.primary_sidebar().stats.0.as_number()
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
