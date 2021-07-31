//! Playlists

pub mod video;

pub use self::video::Video;

use std::sync::Arc;

use crate::{
    youtube::{
        browse::{
            self,
            playlist::{PlaylistSidebarPrimaryInfoRenderer, PlaylistSidebarSecondaryInfoRenderer},
        },
        innertube::Browse,
    },
    Client, Thumbnail,
};

/// A Id describing a Playlist.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Id(String);

impl std::str::FromStr for Id {
    type Err = crate::error::Id<0>;

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
            Ok(Self(id.to_string()))
        } else {
            Err(crate::error::Id::InvalidId(s.to_string()))
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
    client: Arc<Client>,
    response: browse::playlist::Ok,
}

impl Playlist {
    pub(crate) async fn get(client: Arc<crate::Client>, id: Id) -> crate::Result<Self> {
        let response: browse::playlist::Result = client.api.browse(Browse::Playlist(id)).await?;
        let response = response.into_std()?;

        Ok(Self { client, response })
    }

    fn microformat(&self) -> &browse::playlist::MicroformatDataRenderer {
        &self.response.microformat.microformat_data_renderer
    }

    fn primary_sidebar(&self) -> &PlaylistSidebarPrimaryInfoRenderer {
        &self
            .response
            .sidebar
            .playlist_sidebar_renderer
            .items
            .0
            .playlist_sidebar_primary_info_renderer
    }

    fn secondary_sidebar(&self) -> Option<&PlaylistSidebarSecondaryInfoRenderer> {
        self.response
            .sidebar
            .playlist_sidebar_renderer
            .items
            .1
            .as_ref()
            .map(|x| &x.playlist_sidebar_secondary_info_renderer)
    }

    /// The [`Id`] of a playlist
    pub fn id(&self) -> Id {
        self.microformat()
            .url_canonical
            .clone()
            .split_off(37)
            .parse()
            .expect("Id returned from YouTube was not parsable")
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
    pub fn channel(&self) -> Option<Channel<'_>> {
        let sec = &self.secondary_sidebar()?.video_owner.video_owner_renderer;
        Some(Channel {
            client: Arc::clone(&self.client),
            id: sec.id(),
            name: sec.name(),
        })
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
            let mut videos: Box<dyn Iterator<Item = browse::playlist::PlaylistItem>> = Box::new(self.response.contents.videos().cloned());

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

/// The creator of a [`Playlist`]
pub struct Channel<'a> {
    client: Arc<Client>,
    id: crate::channel::Id,
    name: &'a str,
}

impl<'a> Channel<'a> {
    /// The [`Id`](crate::channel::Id) of a [`Channel`]
    pub fn id(&self) -> crate::channel::Id {
        self.id
    }

    /// The name of a [`Channel`]
    pub fn name(&self) -> &str {
        self.name
    }

    /// Refetch the channel to get more information
    pub async fn upgrade(&self) -> crate::Result<crate::Channel> {
        self.client.channel(self.id).await
    }
}

impl<'a> std::fmt::Debug for Channel<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}
