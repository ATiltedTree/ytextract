//! Video types.
//!
//! # Example
//!
//! ```rust
//! # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = youtube_extractor::Client::new().await?;
//!
//! let video = client.video("nI2e-J6fsuk".parse()?).await?;
//!
//! println!("Title: {}", video.title());
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::Error;
use crate::Thumbnail;

static DATA_EXP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"var ytInitialData = (\{.*\});.*</script>").unwrap());
static PLAYER_RESPONSE_EXP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"var ytInitialPlayerResponse = (\{.*\});.*</script>").unwrap());
static YTCFG_EXP: Lazy<Regex> = Lazy::new(|| Regex::new(r"\nytcfg.set\((\{.*\})\);").unwrap());

/// A Video found on YouTube
///
/// # Example
///
/// ```rust
/// # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = youtube_extractor::Client::new().await?;
///
/// let video = client.video("nI2e-J6fsuk".parse()?).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Video {
    client: reqwest::Client,
    initial_data: Value,
    player_response: Value,
    ytcfg: Value,
}

impl Video {
    pub(crate) async fn get(client: reqwest::Client, id: Id) -> crate::Result<Self> {
        let watch_page = client
            .get(format!("https://youtube.com/watch?v={}&hl=en", id))
            .send()
            .await?
            .error_for_status()?;

        let body = watch_page.text().await?;

        let initial_data = DATA_EXP
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or(Error::MissingData)?
            .as_str();
        let initial_data = serde_json::from_str(initial_data)?;

        let player_response = PLAYER_RESPONSE_EXP
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or(Error::MissingData)?
            .as_str();
        let player_response = serde_json::from_str(player_response)?;

        let ytcfg = YTCFG_EXP
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or(Error::MissingData)?
            .as_str();
        let ytcfg = serde_json::from_str(ytcfg)?;

        Ok(Self {
            client,
            initial_data,
            player_response,
            ytcfg,
        })
    }

    /// The title of a [`Video`]
    pub fn title(&self) -> &str {
        &self.player_response["videoDetails"]["title"]
            .as_str()
            .expect("A YouTube title was not a string")
    }

    /// The [`Id`] of a [`Video`]
    pub fn id(&self) -> Id {
        self.player_response["videoDetails"]["videoId"]
            .as_str()
            .expect("A YouTube VideoId was not a string")
            .parse()
            .expect("A YouTube VideoId was not the correct size")
    }

    /// The [`Duration`] of a [`Video`]
    pub fn duration(&self) -> Duration {
        Duration::from_secs(
            self.player_response["videoDetails"]["lengthSeconds"]
                .as_str()
                .expect("A YouTube duration was not found")
                .parse()
                .expect("A YouTube duration was not a integer"),
        )
    }

    /// The keyword/tags of a [`Video`]
    pub fn keywords(&self) -> Option<Vec<&str>> {
        self.player_response["videoDetails"]["keywords"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| v.as_str().expect("A YouTube keyword was not a string"))
                    .collect::<Vec<&str>>()
            })
    }

    /// The [`ChannelId`][crate::channel::Id] of a [`Video`]
    pub fn channel_id(&self) -> crate::channel::Id {
        self.player_response["videoDetails"]["channelId"]
            .as_str()
            .expect("A YouTube ChannelId was not a string")
            .parse()
            .expect("A YouTube ChannelId was not the correct size")
    }

    /// The author of a [`Video`]
    pub fn author(&self) -> &str {
        self.player_response["videoDetails"]["author"]
            .as_str()
            .expect("A YouTube author was not a string")
    }

    /// The description of a [`Video`]
    pub fn description(&self) -> &str {
        self.player_response["videoDetails"]["shortDescription"]
            .as_str()
            .expect("A YouTube description was not a string")
    }

    /// The views of a [`Video`]
    pub fn views(&self) -> u64 {
        self.player_response["videoDetails"]["viewCount"]
            .as_str()
            .expect("A YouTube viewCount was not a string")
            .parse()
            .expect("A YouTube viewCount was not parsable as a unsigned integer")
    }

    /// The [`Ratings`] of a [`Video`]
    pub fn ratings(&self) -> Ratings {
        let allowed = self.player_response["videoDetails"]["allowRatings"]
            .as_bool()
            .expect("allowRatings was not a bool");

        let fixed_tooltip = self.initial_data["contents"]["twoColumnWatchNextResults"]["results"]
            ["results"]["contents"]
            .as_array()
            .expect("InitialData contents was not an array")
            .iter()
            .find_map(|v| v.get("videoPrimaryInfoRenderer"))
            .expect("InitialData contents did not have a videoPrimaryInfoRenderer")["sentimentBar"]
            ["sentimentBarRenderer"]["tooltip"]
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

        Ratings {
            allowed,
            likes,
            dislikes,
        }
    }

    /// If a [`Video`] is private
    pub fn private(&self) -> bool {
        self.player_response["videoDetails"]["isPrivate"]
            .as_bool()
            .expect("isPrivate was not a bool")
    }

    /// If a [`Video`] is live (e.g. a Livestream) or if it was live in the past
    pub fn live(&self) -> bool {
        self.player_response["videoDetails"]["isLiveContent"]
            .as_bool()
            .expect("isLiveContent was not a bool")
    }

    /// The [`Thumbnails`][Thumbnail] of a [`Video`]
    pub fn thumbnails(&self) -> Vec<Thumbnail> {
        self.player_response["videoDetails"]["thumbnail"]["thumbnails"]
            .as_array()
            .expect("A Video did not have any thumbnails")
            .iter()
            .map(Thumbnail::from)
            .collect()
    }

    /// If a [`Video`] is age-restricted. This is the opposite of
    /// [`Video::family_safe`].
    pub fn age_restricted(&self) -> bool {
        !self.family_safe()
    }

    fn microformat(&self) -> &Value {
        &self.player_response["microformat"]["playerMicroformatRenderer"]
    }

    /// If a [`Video`] is family safe
    pub fn family_safe(&self) -> bool {
        self.microformat()["isFamilySafe"]
            .as_bool()
            .expect("isFamilySafe was not a bool!")
    }

    /// If a [`Video`] is unlisted
    pub fn unlisted(&self) -> bool {
        self.microformat()["isUnlisted"]
            .as_bool()
            .expect("isUnlisted was not a bool!")
    }

    /// The category a [`Video`] belongs in
    pub fn category(&self) -> &str {
        self.microformat()["category"]
            .as_str()
            .expect("category was not a string!")
    }

    /// The publish date of a [`Video`]
    pub fn publish_date(&self) -> chrono::NaiveDate {
        self.microformat()["publishDate"]
            .as_str()
            .expect("publishDate was not a string!")
            .parse()
            .expect("publishDate was not parsable as a NaiveDate")
    }

    /// The upload date of a [`Video`]
    pub fn upload_date(&self) -> chrono::NaiveDate {
        self.microformat()["uploadDate"]
            .as_str()
            .expect("uploadDate was not a string!")
            .parse()
            .expect("uploadDate was not parsable as a NaiveDate")
    }
}

/// Ratings on a video
#[derive(Debug)]
pub struct Ratings {
    /// If liking/disliking is allowed on the [`Video`]
    pub allowed: bool,
    /// The amount of likes a [`Video`] received
    pub likes: u64,
    /// The amount of dislikes a [`Video`] received
    pub dislikes: u64,
}

/// A [`Id`][crate::Id] describing a Video.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Id(crate::Id<11>);

/// The [`Error`][std::error::Error] produced when a invalid [`Id`] is
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
    #[error("A VideoId has to be 11 characters long but was {0} long")]
    InvalidLength(usize),
}

impl From<crate::id::Error> for IdError {
    fn from(val: crate::id::Error) -> Self {
        let crate::id::Error { expected: _, found } = val;
        IdError::InvalidLength(found)
    }
}

impl std::str::FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIXES: [&str; 4] = [
            "https://www.youtube.com/watch?v=",
            "https://youtu.be/",
            "https://www.youtube.com/embed/",
            // No Prefix matched. Possibly naked id (OLWUqW4BRl4). Length and
            // correctness will be checked later.
            "",
        ];

        let id = PREFIXES
            .iter()
            .find_map(|prefix| s.strip_prefix(prefix))
            .unwrap();

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
    #[async_std::test]
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
            Some(vec![
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
            ])
        );
        assert_eq!(video.channel_id(), "UCXuqSBlHAE6Xw-yeJA0Tunw".parse()?);
        assert_eq!(video.author(), "Linus Tech Tips");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 1_068_917);

        let ratings = video.ratings();
        assert!(ratings.allowed);
        assert!(ratings.likes >= 51_745);
        assert!(ratings.dislikes >= 622);

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

    #[async_std::test]
    async fn unlisted() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::client::Client::new().await?;

        let video = client
            .video("https://www.youtube.com/watch?v=9Jg_Fwc0QOY".parse()?)
            .await?;

        assert_eq!(video.title(), "youtube_explode_dart test");

        assert_eq!(video.id(), "9Jg_Fwc0QOY".parse()?);
        assert_eq!(video.duration(), std::time::Duration::from_secs(10));
        assert_eq!(video.keywords(), None);
        assert_eq!(video.channel_id(), "UCZqdX9k5eyv1aO7i2746bXg".parse()?);
        assert_eq!(video.author(), "ATiltedTree");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 6);

        let ratings = video.ratings();
        assert!(ratings.allowed);
        assert!(ratings.likes == 0);
        assert!(ratings.dislikes == 0);

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

    #[async_std::test]
    async fn age_restricted() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::client::Client::new().await?;

        let video = client
            .video("https://www.youtube.com/watch?v=uc8BltmHWww".parse()?)
            .await?;

        assert_eq!(video.title(), "LoL emoticonos version nopor");

        assert_eq!(video.id(), "uc8BltmHWww".parse()?);
        assert_eq!(video.duration(), std::time::Duration::from_secs(110));
        assert_eq!(video.keywords(), None);
        assert_eq!(video.channel_id(), "UCNsCnSYsc6RT9LNxysuEwNg".parse()?);
        assert_eq!(video.author(), "lol mas18");
        assert!(!video.description().is_empty());
        assert!(video.views() >= 245_175);

        let ratings = video.ratings();
        assert!(ratings.allowed);
        assert!(ratings.likes >= 2_724);
        assert!(ratings.dislikes >= 164);

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
