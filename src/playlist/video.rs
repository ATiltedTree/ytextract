//! Playlist videos

use std::sync::Arc;

use crate::{youtube::browse, Client, Thumbnail};

/// The reason as to why a [`Video`] is unavailable
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum UnavailabilityReason {
    /// The [`Video`] was deleted
    Deleted,
    /// The [`Video`] was made private
    Private,
}

impl UnavailabilityReason {
    fn from_title(title: impl AsRef<str>) -> Self {
        match title.as_ref() {
            "[Deleted video]" => Self::Deleted,
            "[Private video]" => Self::Private,
            unknown => unimplemented!("Unknown error title: '{}'", unknown),
        }
    }
}

/// A [`Error`](std::error::Error) that occurs when a [`Video`] in a
/// [`Playlist`](super::Playlist) is unavailable
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("Video with id '{id}' is unavailable with reason: '{reason:?}'")]
pub struct Error {
    /// The [`Id`](crate::video::Id) of the unavailable [`Video`]
    pub id: crate::video::Id,
    /// The [`Reason`](UnavailabilityReason) why this [`Video`] is unavailable
    pub reason: UnavailabilityReason,
}

/// A Video of a [`Playlist`](super::Playlist).
#[derive(Clone)]
pub struct Video {
    client: Arc<Client>,
    video: browse::playlist::PlaylistVideo,
}

impl Video {
    pub(super) fn new(
        client: Arc<Client>,
        video: browse::playlist::PlaylistVideoRenderer,
    ) -> Result<Self, Error> {
        match video {
            browse::playlist::PlaylistVideoRenderer::Ok(video) => Ok(Self { client, video }),
            browse::playlist::PlaylistVideoRenderer::Err { title, video_id } => Err(Error {
                id: video_id,
                reason: UnavailabilityReason::from_title(title.runs.0.text),
            }),
        }
    }

    /// The [`Id`](crate::video::Id) of a video.
    pub fn id(&self) -> crate::video::Id {
        self.video.video_id
    }

    /// The title of a video.
    pub fn title(&self) -> &str {
        &self.video.title.runs.0.text
    }

    /// The length of a video.
    pub fn length(&self) -> std::time::Duration {
        self.video.length_seconds
    }

    /// The [`Thumbnails`](Thumbnail) of a video.
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self.video.thumbnail.thumbnails
    }

    /// The author of a video.
    pub fn channel(&self) -> super::Channel<'_> {
        let short = &self.video.short_byline_text.runs.0;
        super::Channel {
            client: Arc::clone(&self.client),
            id: short.navigation_endpoint.browse_endpoint.browse_id,
            name: &short.text,
        }
    }

    /// Refetch this video for more information.
    pub async fn upgrade(&self) -> crate::Result<crate::Video> {
        self.client.video(self.id()).await
    }

    /// Get the [`Streams`](crate::Stream) for this video.
    pub async fn streams(&self) -> crate::Result<impl Iterator<Item = crate::Stream>> {
        self.client.streams(self.id()).await
    }
}

impl std::fmt::Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlaylistVideo")
            .field("id", &self.id())
            .field("title", &self.title())
            .field("length", &self.length())
            .field("thumbnails", &self.thumbnails())
            .field("author", &self.channel())
            .finish()
    }
}

impl PartialEq for Video {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Video {}
