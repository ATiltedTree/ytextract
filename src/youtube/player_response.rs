use std::time::Duration;

use crate::error::Youtube;
use serde::Deserialize;

use super::Thumbnails;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Result<T> {
    Ok(T),
    Error {
        #[serde(rename = "playabilityStatus")]
        playability_status: PlayabilityStatus,
    },
}

impl<T> Result<T> {
    pub fn into_std(self) -> crate::Result<T> {
        match self {
            Self::Error { playability_status } => {
                Err(crate::Error::Youtube(playability_status.as_error()))
            }
            Self::Ok(ok) => Ok(ok),
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub video_details: VideoDetails,
}

#[serde_with::serde_as]
#[derive(Deserialize, Clone)]
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

    pub is_live_content: bool,

    pub thumbnail: Thumbnails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamPlayerResponse {
    pub streaming_data: StreamingData,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    #[serde(default)]
    pub adaptive_formats: Vec<Format>,
}

#[derive(Deserialize, Clone)]
pub struct Format {
    #[serde(flatten)]
    pub base: CommonFormat,
    #[serde(flatten)]
    pub ty: FormatType,
}

#[serde_with::serde_as]
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommonFormat {
    pub url: String,
    pub mime_type: String,
    pub itag: u64,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    #[serde(default)]
    pub content_length: Option<u64>,
    pub bitrate: u64,
    #[serde_as(as = "Option<serde_with::DurationMilliSeconds<String>>")]
    #[serde(default, rename = "approxDurationMs")]
    pub duration: Option<Duration>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum FormatType {
    Audio(AudioFormat),
    Video(VideoFormat),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoFormat {
    pub width: u64,
    pub height: u64,
    pub fps: u64,
    pub quality_label: String,
}

#[serde_with::serde_as]
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioFormat {
    pub loudness_db: Option<f64>,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub audio_sample_rate: u64,
    pub audio_quality: String,
    pub audio_channels: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub reason: String,
}

impl PlayabilityStatus {
    fn as_error(&self) -> crate::error::Youtube {
        match self.reason.as_str() {
            "This video is unavailable" => Youtube::NotFound,
            "This video is no longer available because the YouTube account associated with this video has been terminated." => Youtube::AccountTerminated,
            "The YouTube account associated with this video has been terminated due to multiple third-party notifications of copyright infringement." => Youtube::AccountTerminated,
            "This video has been removed by the uploader" => Youtube::RemovedByUploader,
            "This video has been removed for violating YouTube's policy on nudity or sexual content" => Youtube::NudityOrSexualContentViolation,
            "This video is private" => Youtube::Private,
            "This video has been removed for violating YouTube's Terms of Service." => Youtube::TermsOfServiceViolation,
            "This video has been removed for violating YouTube's Terms of Service" => Youtube::TermsOfServiceViolation,
            "This video has been removed for violating YouTube's Community Guidelines" => Youtube::CommunityGuidelineViolation,
            "This video is no longer available due to a privacy claim by a third party" => Youtube::PrivacyClaim,
            "This video requires payment to watch." => Youtube::PurchaseRequired,
            "This video may be inappropriate for some users." => Youtube::AgeRestricted,
            "This video is not available in your country" => Youtube::GeoRestricted,
            "This video is no longer available because the uploader has closed their YouTube account." => Youtube::AccountDeleted,
            copyright if copyright.starts_with("This video is no longer available due to a copyright claim by") => {
                let who = copyright
                    .strip_prefix("This video is no longer available due to a copyright claim by ")
                    .unwrap();

                Youtube::CopyrightClaim {
                    claiment: who.to_string()
                }
            }
            copyright if copyright.starts_with("This video contains content from ") => {
                let who = copyright
                    .strip_prefix("This video contains content from ")
                    .unwrap()
                    .strip_suffix(", who has blocked it in your country on copyright grounds.")
                    .unwrap();

                Youtube::CopyrightClaim {
                    claiment: who.to_string()
                }
            }
            unknown => unimplemented!("Unknown error reason: `{}`", unknown),
        }
    }
}
