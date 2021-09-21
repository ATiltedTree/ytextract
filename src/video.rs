//! Videos.
//!
//! Videos are identified by a unique [`Id`], and can be queried with a
//! [`Client`](crate::Client).
//!
//! Once you have a [`Video`] you can use [`Video::streams`] to get its
//! [`Streams`](crate::Stream). These contain URLs to download videos,
//! along with metadata about dimensions and fps.
//!
//! # Example
//!
//! ```rust
//! # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ytextract::Client::new();
//!
//! let video = client.video("nI2e-J6fsuk".parse()?).await?;
//!
//! println!("Title: {}", video.title());
//! # Ok(())
//! # }
//! ```

pub mod related;

use crate::{
    youtube::{innertube::Next, next, parse_date, player_response::PlayerResponse},
    Client, Stream, Thumbnail,
};

use std::time::Duration;

/// A Video.
///
/// For more information see the [crate level documentation](crate::video)
#[derive(Clone)]
pub struct Video {
    player_response: PlayerResponse,
    initial_data: next::Root,
    client: Client,
}

impl Video {
    pub(crate) async fn get(client: Client, id: Id) -> crate::Result<Self> {
        Ok(Self {
            player_response: client.api.player(id).await?.into_std()?,
            initial_data: client.api.next(Next::Video(id)).await?,
            client,
        })
    }

    /// The title of a [`Video`].
    pub fn title(&self) -> &str {
        &self.player_response.video_details.title
    }

    /// The [`Id`] of a [`Video`].
    pub fn id(&self) -> Id {
        self.player_response.video_details.video_id
    }

    /// The [`Duration`] of a [`Video`].
    pub fn duration(&self) -> Duration {
        self.player_response.video_details.length_seconds
    }

    /// The keyword/tags of a [`Video`].
    pub fn keywords(&self) -> &Vec<String> {
        &self.player_response.video_details.keywords
    }

    /// The [`Channel`] of a [`Video`].
    pub fn channel(&self) -> Channel<'_> {
        let owner = &self
            .initial_data
            .contents
            .two_column_watch_next_results
            .results
            .results
            .secondary()
            .owner
            .video_owner_renderer;
        Channel {
            client: &self.client,
            id: self.player_response.video_details.channel_id,
            name: &self.player_response.video_details.author,
            subscribers: owner.subscribers(),
            thumbnails: owner.thumbnails(),
        }
    }

    /// The description of a [`Video`].
    pub fn description(&self) -> &str {
        &self.player_response.video_details.short_description
    }

    /// The amount of views a [`Video`] received.
    pub fn views(&self) -> u64 {
        self.player_response.video_details.view_count
    }

    /// The [`Ratings`] a [`Video`] received.
    pub fn ratings(&self) -> Ratings {
        if let Some((likes, dislikes)) = self
            .initial_data
            .contents
            .two_column_watch_next_results
            .results
            .results
            .primary()
            .ratings()
        {
            Ratings::Allowed { likes, dislikes }
        } else {
            Ratings::NotAllowed
        }
    }

    /// The hashtags a [`Video`] is tagged with.
    pub fn hashtags(&self) -> impl Iterator<Item = &str> {
        self.initial_data
            .contents
            .two_column_watch_next_results
            .results
            .results
            .primary()
            .hashtags()
    }

    /// If a [`Video`] is live (e.g. a Livestream) or if it was live in the
    /// past.
    pub fn live(&self) -> bool {
        self.player_response.video_details.is_live_content
    }

    /// The [`Thumbnails`](Thumbnail) of a [`Video`]
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self.player_response.video_details.thumbnail.thumbnails
    }

    /// The date a [`Video`] was published.
    pub fn date(&self) -> chrono::NaiveDate {
        parse_date(
            &self
                .initial_data
                .contents
                .two_column_watch_next_results
                .results
                .results
                .primary()
                .date_text,
        )
        .expect("Unable to parse date")
    }

    /// The [`Items`](Related) related to a [`Video`].
    pub fn related(&self) -> impl futures_core::Stream<Item = Related> {
        let initial_items = self
            .initial_data
            .contents
            .two_column_watch_next_results
            .secondary_results
            .secondary_results
            .results
            .clone();
        let client = self.client.clone();

        async_stream::stream! {
            let mut items: Box<dyn Iterator<Item = next::RelatedItem> + Send + Sync> =
                Box::new(initial_items.into_iter());

            while let Some(item) = items.next() {
                match item {
                    next::RelatedItem::ContinuationItemRenderer(continuation) => {
                        assert!(
                            items.next().is_none(),
                            "Found a continuation in the middle of items!"
                        );
                        let response: next::Continuation = client
                            .api
                            .next(Next::Continuation(continuation.get()))
                            .await
                            .expect("Continuation request failed");

                        items = Box::new(response.into_videos());
                    }
                    next::RelatedItem::CompactVideoRenderer(video) => {
                        yield Related::Video(related::Video(video, client.clone()));
                    }
                    next::RelatedItem::CompactPlaylistRenderer(playlist) => {
                        yield Related::Playlist(related::Playlist(playlist, client.clone()));
                    }
                    next::RelatedItem::CompactRadioRenderer(radio) => {
                        yield Related::Radio(related::Radio(radio, client.clone()));
                    }
                    next::RelatedItem::CompactMovieRenderer(movie) => {
                        yield Related::Movie(related::Movie(movie, client.clone()));
                    },
                    // I don't know what this is - just skip it
                    next::RelatedItem::PromotedSparklesWebRenderer {} => continue,
                }
            }
        }
    }

    /// The [`Streams`](Stream) of a [`Video`]
    pub async fn streams(&self) -> crate::Result<impl Iterator<Item = Stream>> {
        crate::stream::get(self.client.clone(), self.id()).await
    }
}

impl std::fmt::Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Video")
            .field("id", &self.id())
            .field("title", &self.title())
            .field("duration", &self.duration())
            .field("keywords", &self.keywords())
            .field("channel", &self.channel())
            .field("description", &self.description())
            .field("views", &self.views())
            .field("ratings", &self.ratings())
            .field("live", &self.live())
            .field("thumbnails", &self.thumbnails())
            .field("date", &self.date())
            .finish()
    }
}

impl PartialEq for Video {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Video {}

/// The uploader of a video
pub struct Channel<'a> {
    client: &'a Client,
    id: crate::channel::Id,
    name: &'a str,
    subscribers: Option<u64>,
    thumbnails: &'a Vec<Thumbnail>,
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

    /// The amount of subscribers a [`Channel`] has.
    pub fn subscribers(&self) -> Option<u64> {
        self.subscribers
    }

    /// The [`Thumbnails`](Thumbnail) of a [`Channel`]
    pub fn thumbnails(&self) -> impl Iterator<Item = &Thumbnail> {
        self.thumbnails.iter()
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
            .field("subscribers", &self.subscribers)
            .field("thumbnails", &self.thumbnails)
            .finish()
    }
}

impl<'a> PartialEq for Channel<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<'a> Eq for Channel<'a> {}

/// Ratings on a video
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ratings {
    /// Rating is allowed
    Allowed {
        /// The amount of likes a [`Video`] received
        likes: u64,
        /// The amount of dislikes a [`Video`] received
        dislikes: u64,
    },

    /// Rating is not allowed
    NotAllowed,
}

define_id! {
    11,
    "An Id describing a [`Video`]",
    [
        "https://www.youtube.com/watch?v=",
        "https://youtu.be/",
        "https://www.youtube.com/embed/",
    ]
}

/// A Item that is related to a [`Video`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Related {
    /// A Video
    Video(related::Video),

    /// A Playlist
    Playlist(related::Playlist),

    /// A Movie
    Movie(related::Movie),

    /// A Radio
    Radio(related::Radio),
}
