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

pub use self::audio::Stream as Audio;
pub use self::common::Stream as Common;
pub use self::video::Stream as Video;
pub use crate::youtube::player_response::Quality;
use crate::{
    youtube::player_response::{FormatType, StreamingData},
    Client,
};
use reqwest::Url;
use std::{collections::HashMap, sync::Arc};

pub(crate) async fn get(
    client: Arc<Client>,
    id: crate::video::Id,
    streaming_data: Option<StreamingData>,
) -> crate::Result<impl Iterator<Item = Stream>> {
    let streaming_data = if let Some(streaming_data) = streaming_data {
        streaming_data
    } else {
        let response = match client.api.player(id).await.unwrap().into_std() {
            Ok(response) if response.is_streamable() => response.streaming_data,
            _ => Some(
                client
                    .api
                    .get_video_info(id)
                    .await
                    .unwrap()
                    .into_std()
                    .unwrap(),
            ),
        };

        response.expect("Recoverable error did not contain streaming data")
    };

    let needs_player = streaming_data
        .adaptive_formats
        .iter()
        .any(|f| f.base.url.is_none());

    if needs_player {
        client.init_player().await;
    }

    // TODO: DashManifest/HlsManifest
    Ok(streaming_data
        .adaptive_formats
        .into_iter()
        .map(move |stream| Stream::new(stream, Arc::clone(&client))))
}

/// A Stream of a YouTube video
pub enum Stream {
    /// A [`Audio`]
    Audio(Audio),
    /// A [`Video`]
    Video(Video),
}

impl Stream {
    pub(crate) fn new(
        format: crate::youtube::player_response::Format,
        client: Arc<Client>,
    ) -> Self {
        match format.ty {
            FormatType::Audio(audio) => Self::Audio(Audio {
                common: Common {
                    url: Stream::resolve_url(&client, &format.base),
                    format: format.base,
                    client,
                },
                audio,
            }),
            FormatType::Video(video) => Self::Video(Video {
                common: Common {
                    url: Stream::resolve_url(&client, &format.base),
                    format: format.base,
                    client,
                },
                video,
            }),
        }
    }

    fn resolve_url(
        client: &Arc<Client>,
        format: &crate::youtube::player_response::CommonFormat,
    ) -> Url {
        match &format.url {
            Some(url) => url.clone(),
            None => {
                let signature_cipher = format
                    .signature_cipher
                    .as_ref()
                    .expect("Stream did not have a URL or signatureCipher");
                let root: HashMap<String, String> =
                    serde_urlencoded::from_str(signature_cipher.as_str())
                        .expect("signatureCipher was not urlencoded");

                let signature = client.player().cipher().run(root["s"].clone());
                let signature_arg = &root["sp"];
                let mut url = Url::parse(&root["url"])
                    .expect("signatureCipher url attribute was not a valid URL");

                let query = url
                    .query()
                    .map(|q| format!("{}&{}={}", q, signature_arg, signature))
                    .expect("URL did not have a query");

                url.set_query(Some(&query));
                url
            }
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
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        match self {
            Stream::Audio(audio) => &audio.common,
            Stream::Video(video) => &video.common,
        }
    }
}

impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Stream");

        match self {
            Stream::Audio(audio) => {
                audio.common.debug(&mut debug);
                audio.debug(&mut debug);
            }
            Stream::Video(video) => {
                video.common.debug(&mut debug);
                video.debug(&mut debug);
            }
        }
        debug.finish()?;

        Ok(())
    }
}
