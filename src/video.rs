//! Video types.
//!
//! # Example
//!
//! ```rust
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ytextract::Client::new().await?;
//!
//! let video = client.video("nI2e-J6fsuk".parse()?).await?;
//!
//! println!("Title: {}", video.title());
//! # Ok(())
//! # }
//! ```

pub use crate::youtube::player_response::PlayabilityErrorCode;

use crate::{
    youtube::player_response::{Microformat, StreamingData, VideoDetails},
    Client, Stream, Thumbnail,
};

use serde_json::Value;

use std::{sync::Arc, time::Duration};

/// A Error that occurs when querying a [`Video`](crate::Video).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`Video`] is unplayable due to a YouTube error
    #[error("{code:?}: '{reason:?}'")]
    Unplayable {
        /// The [`PlayabilityErrorCode`] returned by YouTube for processing
        code: PlayabilityErrorCode,
        /// The optional Human-readable reason for the error
        reason: Option<String>,
    },
}

/// A Video found on YouTube
///
/// # Example
///
/// ```rust
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ytextract::Client::new().await?;
///
/// let video = client.video("nI2e-J6fsuk".parse()?).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Video {
    initial_data: Value,
    video_details: VideoDetails,
    microformat: Microformat,
    streaming_data: Option<StreamingData>,
    client: Arc<Client>,
}

impl Video {
    pub(crate) async fn get(client: Arc<Client>, id: Id) -> crate::Result<Self> {
        let player_response = client.api.player(id).await?;

        if player_response.playability_status.status.is_recoverable() {
            Ok(Self {
                initial_data: client.api.next(id).await?,
                video_details: player_response
                    .video_details
                    .expect("Recoverable error did not contain video_details"),
                microformat: player_response
                    .microformat
                    .expect("Recoverable error did not contain microformat"),
                client,
                streaming_data: player_response.streaming_data,
            })
        } else {
            Err(Error::Unplayable {
                code: player_response.playability_status.status,
                reason: player_response.playability_status.reason,
            }
            .into())
        }
    }

    /// The title of a [`Video`]
    pub fn title(&self) -> &str {
        &self.video_details.title
    }

    /// The [`Id`] of a [`Video`]
    pub fn id(&self) -> Id {
        self.video_details.video_id
    }

    /// The [`Duration`] of a [`Video`]
    pub fn duration(&self) -> Duration {
        self.video_details.length_seconds
    }

    /// The keyword/tags of a [`Video`]
    pub fn keywords(&self) -> &Vec<String> {
        &self.video_details.keywords
    }

    /// The [`ChannelId`](crate::channel::Id) of a [`Video`]
    pub fn channel_id(&self) -> crate::channel::Id {
        self.video_details.channel_id
    }

    /// The author of a [`Video`]
    pub fn author(&self) -> &str {
        &self.video_details.author
    }

    /// The description of a [`Video`]
    pub fn description(&self) -> &str {
        &self.video_details.short_description
    }

    /// The views of a [`Video`]
    pub fn views(&self) -> u64 {
        self.video_details.view_count
    }

    /// The [`Ratings`] of a [`Video`]
    pub fn ratings(&self) -> Ratings {
        if self.video_details.allow_ratings {
            let fixed_tooltip = self.initial_data["contents"]["twoColumnWatchNextResults"]
                ["results"]["results"]["contents"]
                .as_array()
                .expect("InitialData contents was not an array")
                .iter()
                .find_map(|v| v.get("videoPrimaryInfoRenderer"))
                .expect("InitialData contents did not have a videoPrimaryInfoRenderer")
                ["sentimentBar"]["sentimentBarRenderer"]["tooltip"]
                .as_str()
                .expect("sentimentBar tooltip was not a string")
                .replace(',', "");
            let (likes, dislikes) = fixed_tooltip
                .split_once(" / ")
                .expect("sentimentBar tooltip did not have a '/'");

            let likes = likes
                .parse()
                .expect("Likes we not parsable as a unsigned integer");
            let dislikes = dislikes
                .parse()
                .expect("Dislikes we not parsable as a unsigned integer");

            Ratings::Allowed { likes, dislikes }
        } else {
            Ratings::NotAllowed
        }
    }

    /// If a [`Video`] is private
    pub fn private(&self) -> bool {
        self.video_details.is_private
    }

    /// If a [`Video`] is live (e.g. a Livestream) or if it was live in the past
    pub fn live(&self) -> bool {
        self.video_details.is_live_content
    }

    /// The [`Thumbnails`](Thumbnail) of a [`Video`]
    pub fn thumbnails(&self) -> &Vec<Thumbnail> {
        &self.video_details.thumbnail.thumbnails
    }

    /// If a [`Video`] is age-restricted. This is the opposite of
    /// [`Video::family_safe`].
    pub fn age_restricted(&self) -> bool {
        !self.family_safe()
    }

    fn microformat(&self) -> &crate::youtube::player_response::PlayerMicroformatRenderer {
        &self.microformat.player_microformat_renderer
    }

    /// If a [`Video`] is family safe
    pub fn family_safe(&self) -> bool {
        self.microformat().is_family_safe
    }

    /// If a [`Video`] is unlisted
    pub fn unlisted(&self) -> bool {
        self.microformat().is_unlisted
    }

    /// The category a [`Video`] belongs in
    pub fn category(&self) -> &str {
        &self.microformat().category
    }

    /// The publish date of a [`Video`]
    pub fn publish_date(&self) -> chrono::NaiveDate {
        self.microformat().publish_date
    }

    /// The upload date of a [`Video`]
    pub fn upload_date(&self) -> chrono::NaiveDate {
        self.microformat().upload_date
    }

    /// The [`Stream`]s of a [`Video`]
    pub async fn streams(&self) -> crate::Result<impl Iterator<Item = Stream>> {
        crate::stream::get(
            Arc::clone(&self.client),
            self.id(),
            self.streaming_data.clone(),
        )
        .await
    }
}

/// Ratings on a video
#[derive(Debug)]
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

/// A [`Id`](crate::Id) describing a Video.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Id(crate::Id<11>);

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

    /// A [`Id`] had an invalid length. All [`Id`]s have to be 11 characters
    /// long
    #[error(transparent)]
    InvalidLength(#[from] crate::id::Error),
}

impl std::str::FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIXES: [&str; 3] = [
            "https://www.youtube.com/watch?v=",
            "https://youtu.be/",
            "https://www.youtube.com/embed/",
        ];

        let id = PREFIXES
            .iter()
            .find_map(|prefix| s.strip_prefix(prefix))
            // No Prefix matched. Possibly naked id (OLWUqW4BRl4). Length and
            // correctness will be checked later.
            .unwrap_or(s);

        if id.chars().all(crate::id::validate_char) {
            Ok(Self(id.parse()?))
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

#[cfg(test)]
mod test {
    use crate::video::Ratings;

    #[tokio::test]
    async fn get() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::client::Client::new().await?;

        let video = client
            .video("https://www.youtube.com/watch?v=7B2PIVSWtJA".parse()?)
            .await?;

        assert_eq!(
            video.title(),
            "I Sent Corridor Digital the WORST VFX Workstation"
        );

        assert_eq!(video.id(), "7B2PIVSWtJA".parse()?);
        assert_eq!(video.duration(), std::time::Duration::from_secs(1358));
        assert_eq!(
            video.keywords(),
            &vec![
                "photoshop",
                "adobe",
                "1.0",
                "macintosh",
                "apple",
                "lc",
                "475",
                "quadra",
                "performa",
                "classic",
                "system 7.5",
                "macos",
                "ossc",
                "vga",
                "vfx",
                "editing",
                "challenge",
                "corridor digital",
                "collab",
                "ftp",
                "fetch",
                "icab",
                "marathon",
                "oregon trail",
                "nightmare fuel",
                "scsi2sd"
            ]
        );
        assert_eq!(video.channel_id(), "UCXuqSBlHAE6Xw-yeJA0Tunw".parse()?);
        assert_eq!(video.author(), "Linus Tech Tips");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 1_068_917);

        let ratings = video.ratings();
        if let Ratings::Allowed { likes, dislikes } = ratings {
            assert!(likes >= 51_745);
            assert!(dislikes >= 622);
        } else {
            unreachable!();
        }

        assert!(!video.private());
        assert!(!video.live());
        assert!(!video.thumbnails().is_empty());
        assert!(!video.age_restricted());
        assert!(!video.unlisted());
        assert!(video.family_safe());
        assert_eq!(video.category(), "Science & Technology");
        assert_eq!(
            video.publish_date(),
            chrono::NaiveDate::from_ymd(2021, 4, 14)
        );
        assert_eq!(
            video.upload_date(),
            chrono::NaiveDate::from_ymd(2021, 4, 14)
        );

        Ok(())
    }

    #[tokio::test]
    async fn unlisted() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::client::Client::new().await?;

        let video = client
            .video("https://www.youtube.com/watch?v=9Jg_Fwc0QOY".parse()?)
            .await?;

        assert_eq!(video.title(), "youtube_explode_dart test");

        assert_eq!(video.id(), "9Jg_Fwc0QOY".parse()?);
        assert_eq!(video.duration(), std::time::Duration::from_secs(10));
        assert_eq!(video.keywords(), &Vec::<String>::new());
        assert_eq!(video.channel_id(), "UCZqdX9k5eyv1aO7i2746bXg".parse()?);
        assert_eq!(video.author(), "ATiltedTree");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 6);

        let ratings = video.ratings();
        if let Ratings::Allowed { likes, dislikes } = ratings {
            assert!(likes == 0);
            assert!(dislikes == 0);
        } else {
            unreachable!();
        }

        assert!(!video.private());
        assert!(!video.live());
        assert!(!video.thumbnails().is_empty());
        assert!(!video.age_restricted());
        assert!(video.unlisted());
        assert!(video.family_safe());
        assert_eq!(video.category(), "Science & Technology");
        assert_eq!(
            video.publish_date(),
            chrono::NaiveDate::from_ymd(2021, 3, 14)
        );
        assert_eq!(
            video.upload_date(),
            chrono::NaiveDate::from_ymd(2021, 3, 14)
        );

        Ok(())
    }

    #[tokio::test]
    async fn age_restricted() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::client::Client::new().await?;

        let video = client
            .video("https://www.youtube.com/watch?v=uc8BltmHWww".parse()?)
            .await?;

        assert_eq!(video.title(), "LoL emoticonos version nopor");

        assert_eq!(video.id(), "uc8BltmHWww".parse()?);
        assert_eq!(video.duration(), std::time::Duration::from_secs(110));
        assert_eq!(video.keywords(), &Vec::<String>::new());
        assert_eq!(video.channel_id(), "UCNsCnSYsc6RT9LNxysuEwNg".parse()?);
        assert_eq!(video.author(), "lol mas18");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 245_175);

        let ratings = video.ratings();
        if let Ratings::Allowed { likes, dislikes } = ratings {
            assert!(likes >= 2_724);
            assert!(dislikes >= 164);
        } else {
            unreachable!();
        }

        assert!(!video.private());
        assert!(!video.live());
        assert!(!video.thumbnails().is_empty());
        assert!(video.age_restricted());
        assert!(!video.unlisted());
        assert!(!video.family_safe());
        assert_eq!(video.category(), "People & Blogs");
        assert_eq!(
            video.publish_date(),
            chrono::NaiveDate::from_ymd(2019, 10, 8)
        );
        assert_eq!(
            video.upload_date(),
            chrono::NaiveDate::from_ymd(2019, 10, 8)
        );

        Ok(())
    }
}
