//! Streams of a YouTube video
//!
//! # Example
//!
//! ```rust
//! # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ytextract::Client::new().await?;
//!
//! let streams = client.streams("nI2e-J6fsuk".parse()?).await?;
//!
//! for stream in streams {
//!     println!("Duration: {:?}", stream.duration())
//! }
//!
//! # Ok(())
//! # }
//! ```

mod audio;
mod common;
mod video;

pub use crate::youtube::player_response::Quality;
pub use audio::Stream as AudioStream;
pub use common::Stream as CommonStream;
pub use video::Stream as VideoStream;

use crate::{
    youtube::{
        player_response::{FormatType, PlayabilityErrorCode, StreamingData},
        video_info::VideoInfo,
    },
    Client,
};
use std::sync::Arc;

/// A Error that can occur when working with [`Stream`]s
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Unable to get the content-length of a [`Stream`]
    #[error("Unable to get content-length")]
    UnknownContentLength,

    /// Streams are unplayable due to a YouTube error
    #[error("{code:?}: '{reason:?}'")]
    Unplayable {
        /// The [`PlayabilityErrorCode`] returned by YouTube for processing
        code: PlayabilityErrorCode,
        /// The optional Human-readable reason for the error
        reason: Option<String>,
    },
}

pub(crate) async fn get(
    client: Arc<Client>,
    id: crate::video::Id,
    streaming_data: Option<StreamingData>,
) -> crate::Result<impl Iterator<Item = Stream>> {
    let streaming_data = if let Some(streaming_data) = streaming_data {
        Ok(streaming_data)
    } else {
        let video_info = VideoInfo::from_id(&client.client, id).await?;

        let player_response = video_info.player_response();
        if player_response
            .playability_status
            .status
            .is_stream_recoverable()
        {
            Ok(player_response
                .streaming_data
                .expect("Recoverable error did not contain streaming data"))
        } else {
            Err(Error::Unplayable {
                code: player_response.playability_status.status,
                reason: player_response.playability_status.reason,
            })
        }
    }?;

    // FIXME: DashManifest/HlsManifest
    Ok(streaming_data
        .adaptive_formats
        .into_iter()
        .map(move |stream| Stream::new(stream, Arc::clone(&client))))
}

/// A Stream of a YouTube video
#[derive(Debug)]
pub enum Stream {
    /// A [`AudioStream`]
    Audio(AudioStream),
    /// A [`VideoStream`]
    Video(VideoStream),
}

impl Stream {
    pub(crate) fn new(
        format: crate::youtube::player_response::Format,
        client: Arc<Client>,
    ) -> Self {
        match format.ty {
            FormatType::Audio(audio) => Self::Audio(AudioStream {
                common: CommonStream {
                    format: format.base,
                    client,
                },
                audio,
            }),
            FormatType::Video(video) => Self::Video(VideoStream {
                common: CommonStream {
                    format: format.base,
                    client,
                },
                video,
            }),
        }
    }

    /// Returns `true` if the stream is [`Self::Audio`].
    pub fn is_audio(&self) -> bool {
        matches!(self, Self::Audio(..))
    }

    /// Returns `true` if the stream is [`Self::Video`].
    pub fn is_video(&self) -> bool {
        matches!(self, Self::Video(..))
    }
}

impl std::ops::Deref for Stream {
    type Target = CommonStream;

    fn deref(&self) -> &Self::Target {
        match self {
            Stream::Audio(audio) => &audio.common,
            Stream::Video(video) => &video.common,
        }
    }
}

#[cfg(test)]
mod test {
    #[async_std::test]
    async fn get() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::Client::new().await?;

        let mut streams = client
            .streams("https://www.youtube.com/watch?v=7B2PIVSWtJA".parse()?)
            .await?;

        assert!(streams.next().is_some());

        Ok(())
    }
    #[async_std::test]
    async fn age_restricted() -> Result<(), Box<dyn std::error::Error>> {
        let client = crate::Client::new().await?;

        let streams = client
            .streams("https://www.youtube.com/watch?v=9Jg_Fwc0QOY".parse()?)
            .await?;

        for stream in streams {
            stream.url().await?;
        }

        Ok(())
    }
}
