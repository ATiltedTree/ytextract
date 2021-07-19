//! Playlist videos

use std::sync::Arc;

use crate::{youtube::browse, Client, Thumbnail};

/// The reason as to why a [`Video`] is unavailable
#[derive(Debug)]
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
            unknown => unreachable!("Unknown error title: '{}'", unknown),
        }
    }
}

/// A [`Error`](std::error::Error) that occurs when a [`Video`] in a
/// [`Playlist`](super::Playlist) is unavailable
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[error("Video with id '{id}' is unavailable with reason: '{reason:?}'")]
pub struct Error {
    /// The [`Id`](crate::video::Id) of the unavailable [`Video`]
    pub id: crate::video::Id,
    /// The [`Reason`](UnavailabilityReason) why this [`Video`] is unavailable
    pub reason: UnavailabilityReason,
}

/// A Video of a [`Playlist`](super::Playlist).
pub struct Video {
    client: Arc<Client>,
    video: browse::playlist::PlaylistVideoRenderer,
}

impl Video {
    pub(super) fn new(
        client: Arc<Client>,
        video: browse::playlist::PlaylistVideoRenderer,
    ) -> Result<Self, Error> {
        if video.is_playable {
            Ok(Self { client, video })
        } else {
            Err(Error {
                id: video.video_id,
                reason: UnavailabilityReason::from_title(video.title.runs.0.text),
            })
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
        self.video.length_seconds.expect("No Length")
    }

    /// The [`Thumbnails`](Thumbnail) of a video.
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self.video.thumbnail.thumbnails
    }

    /// Is this video playable/available?
    pub fn playable(&self) -> bool {
        self.video.is_playable
    }

    /// The author of a video.
    pub fn channel(&self) -> super::Channel<'_> {
        let short = &self
            .video
            .short_byline_text
            .as_ref()
            .expect("No channel")
            .runs
            .0;
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
            .field("playable", &self.playable())
            .field("author", &self.channel())
            .finish()
    }
}
