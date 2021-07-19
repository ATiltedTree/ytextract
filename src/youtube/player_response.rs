use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum StreamResult {
    Ok {
        #[serde(rename = "streamingData")]
        streaming_data: StreamingData,
    },
    Error {
        #[serde(rename = "playabilityStatus")]
        playability_status: PlayabilityStatus,
    },
}

impl StreamResult {
    pub fn into_std(self) -> crate::Result<StreamingData> {
        match self {
            Self::Error { playability_status } => {
                Err(crate::Error::Youtube(playability_status.as_error()))
            }
            Self::Ok { streaming_data } => Ok(streaming_data),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Result {
    Ok(PlayerResponse),
    Error {
        #[serde(rename = "playabilityStatus")]
        playability_status: PlayabilityStatus,
    },
}

impl Result {
    pub fn into_std(self) -> crate::Result<PlayerResponse> {
        match self {
            Self::Error { playability_status } => {
                Err(crate::Error::Youtube(playability_status.as_error()))
            }
            Self::Ok(ok) => Ok(ok),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub streaming_data: Option<StreamingData>,
    pub video_details: VideoDetails,
    pub microformat: Microformat,
    pub playability_status: PlayabilityStatus,
}

impl PlayerResponse {
    pub fn is_streamable(&self) -> bool {
        self.playability_status.status == "OK"
    }
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub title: String,
    pub video_id: crate::video::Id,
    #[serde_as(as = "serde_with::DurationSeconds<String>")]
    pub length_seconds: Duration,

    #[serde(default)]
    pub keywords: Vec<String>,
    pub channel_id: crate::channel::Id,

    pub author: String,
    pub short_description: String,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub view_count: u64,

    pub allow_ratings: bool,

    pub is_private: bool,
    pub is_live_content: bool,

    pub thumbnail: Thumbnail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub thumbnails: Vec<crate::Thumbnail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub player_microformat_renderer: PlayerMicroformatRenderer,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMicroformatRenderer {
    // Nonexistant == not family safe
    #[serde(default)]
    pub is_family_safe: bool,
    pub is_unlisted: bool,
    pub category: String,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub publish_date: chrono::NaiveDate,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub upload_date: chrono::NaiveDate,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    #[serde(default)]
    pub adaptive_formats: Vec<Format>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Format {
    #[serde(flatten)]
    pub base: CommonFormat,
    #[serde(flatten)]
    pub ty: FormatType,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommonFormat {
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    #[serde(default)]
    pub url: Option<Url>,
    pub signature_cipher: Option<String>,
    pub quality: Quality,
    pub projection_type: String,
    pub mime_type: String,
    #[serde_as(as = "serde_with::TimestampMilliSeconds<String>")]
    pub last_modified: DateTime<Utc>,
    pub itag: u64,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    #[serde(default)]
    pub content_length: Option<u64>,
    pub bitrate: u64,
    pub average_bitrate: Option<u64>,
    #[serde_as(as = "Option<serde_with::DurationMilliSeconds<String>>")]
    #[serde(default, rename = "approxDurationMs")]
    pub duration: Option<Duration>,

    pub init_range: Option<Range>,
    pub index_range: Option<Range>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum FormatType {
    Audio(AudioFormat),
    Video(VideoFormat),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoFormat {
    pub width: u64,
    pub height: u64,
    pub fps: u64,
    pub quality_label: String,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioFormat {
    pub loudness_db: Option<f64>,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub audio_sample_rate: u64,
    pub audio_quality: String,
    pub audio_channels: u64,
}

#[serde_with::serde_as]
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub start: u64,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub end: u64,
}

/// The quality of a Stream
#[allow(missing_docs)]
#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Quality {
    Tiny,
    Small,
    Medium,
    Large,

    HD720,
    HD1080,
    HD1440,
    HD2160,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub status: String,
    pub error_screen: Option<ErrorScreen>,
}

impl PlayabilityStatus {
    fn as_error(&self) -> crate::error::Youtube {
        use crate::error::Youtube;

        let err = self
            .error_screen
            .as_ref()
            .expect("Error did not have an Error screen")
            .player_error_message_renderer
            .as_ref();

        let err = if let Some(err) = err {
            err
        } else {
            return Youtube::PurchaseRequired;
        };

        match err.reason.simple_text.as_str() {
            "Video unavailable" => Youtube::NotFound,
            "Private video" => Youtube::Private,
            "This video has been removed for violating YouTube's Community Guidelines." => {
                Youtube::CommunityGuidelineViolation
            }
            e => unimplemented!("Unknown error screen text: '{}'", e),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorScreen {
    pub player_error_message_renderer: Option<PlayerErrorMessageRenderer>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerErrorMessageRenderer {
    pub reason: SimpleText,
    //pub subreason: Option<SimpleText>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleText {
    pub simple_text: String,
}
