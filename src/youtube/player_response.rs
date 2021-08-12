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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub streaming_data: Option<StreamingData>,
    pub video_details: VideoDetails,
    pub microformat: Microformat,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub thumbnails: Vec<crate::Thumbnail>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub player_microformat_renderer: PlayerMicroformatRenderer,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
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
    pub reason: Option<String>,
    pub error_screen: Option<ErrorScreen>,
}

impl PlayabilityStatus {
    fn as_error(&self) -> crate::error::Youtube {
        use crate::error::Youtube;

        match (self.reason.as_ref(), self.error_screen.as_ref()) {
            (_, Some(ErrorScreen { player_error_message_renderer: Some(renderer)})) => {
                match renderer.reason.simple_text.as_str() {
                    "Private video" => Youtube::Private,
                    "Video unavailable" => match renderer.subreason().unwrap_or("This video is unavailable") {
                        "This video is no longer available because the YouTube account associated with this video has been terminated." => Youtube::AccountTerminated,
                        "This video is unavailable" => Youtube::NotFound,
                        "This video has been removed by the uploader" => Youtube::RemovedByUploader,
                        "This video is no longer available due to a copyright claim by " => {
                            if let Some(Subreason::Runs(runs)) = &renderer.subreason {
                                let claiment = runs.runs[1].text.clone();
                                Youtube::CopyrightClaim {claiment}
                            } else {
                                unreachable!("copyright claim error screen was not Runs")
                            }
                        }
                        e => unimplemented!("Unknown subreason for video unavailable: '{}'", e),
                    },
                    "Sign in to confirm your age" => Youtube::AgeRestricted,
                    "This video has been removed for violating YouTube's policy on nudity or sexual content." => Youtube::NudityOrSexualContentViolation,
                    "This video has been removed for violating YouTube's Terms of Service." => Youtube::TermsOfServiceViolation,
                    "This video requires payment to watch." => Youtube::PurchaseRequired,
                    e => unimplemented!("Unknown error screen text: '{}'", e)
                }
            },
            (Some(reason), _) => {
                match reason.as_str() {
                    "This video requires payment to watch." => Youtube::PurchaseRequired,
                    e => unimplemented!("Unknown error reason: '{}'", e),
                }
            },
            e => unimplemented!("Got {:#?}, and status '{}'", e, self.status),
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
    subreason: Option<Subreason>,
}

impl PlayerErrorMessageRenderer {
    fn subreason(&self) -> Option<&str> {
        match self.subreason.as_ref()? {
            Subreason::SimpleText(text) => Some(text.simple_text.as_str()),
            Subreason::Runs(runs) => Some(runs.runs.get(0)?.text.as_str()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Subreason {
    SimpleText(SimpleText),
    Runs(Runs),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleText {
    pub simple_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runs {
    pub runs: Vec<TitleRun>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleRun {
    pub text: String,
}
