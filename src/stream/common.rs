use crate::{youtube::player_response::CommonFormat, Client};

use chrono::{DateTime, Utc};
use reqwest::Url;

use std::time::Duration;

/// A [`Stream`](super::Stream) containing video or audio data.
#[derive(Clone)]
pub struct Stream {
    pub(super) format: CommonFormat,
    pub(super) client: Client,
}

impl Stream {
    /// The [`Url`] of a [`Stream`]
    pub fn url(&self) -> Url {
        self.format.url.clone()
    }

    /// The length of a [`Stream`] in bytes
    pub async fn content_length(&self) -> crate::Result<u64> {
        if let Some(content_length) = self.format.content_length {
            Ok(content_length)
        } else {
            let res = self
                .client
                .api
                .http
                .head(self.url())
                .send()
                .await?
                .error_for_status()?;

            Ok(res
                .content_length()
                .expect("HEAD request did not have a content-length"))
        }
    }

    /// Get the [`Stream`] as a [`AsyncStream`](futures_core::Stream) of [`Bytes`](bytes::Bytes)
    pub async fn get(
        &self,
    ) -> crate::Result<impl futures_core::Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
        Ok(self
            .client
            .api
            .http
            .get(self.url())
            .send()
            .await?
            .error_for_status()?
            .bytes_stream())
    }

    /// The [mime type](https://en.wikipedia.org/wiki/Media_type) of a [`Stream`]
    pub fn mime_type(&self) -> &str {
        &self.format.mime_type
    }

    /// The [`DateTime<Utc>`] of when a [`Stream`] was last modified
    pub fn last_modified(&self) -> DateTime<Utc> {
        self.format.last_modified
    }

    /// The bitrate of a [`Stream`]
    pub fn bitrate(&self) -> u64 {
        self.format.bitrate
    }

    /// The [`Duration`] of a [`Stream`]
    pub fn duration(&self) -> Option<Duration> {
        self.format.duration
    }

    pub(super) fn debug(&self, debug: &mut std::fmt::DebugStruct<'_, '_>) {
        debug
            .field("url", &self.url())
            .field("mime_type", &self.mime_type())
            .field("last_modified", &self.last_modified())
            .field("bitrate", &self.bitrate())
            .field("duration", &self.duration());
    }
}

impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("CommonStream");
        self.debug(&mut debug);
        debug.finish()
    }
}
