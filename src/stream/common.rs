use crate::{youtube::player_response::CommonFormat, Client};

use std::time::Duration;
use std::str::FromStr;
use std::io::Read;

/// A [`Stream`](super::Stream) containing video or audio data.
#[derive(Clone)]
pub struct Stream {
    pub(super) format: CommonFormat,
    pub(super) client: Client,
}

impl Stream {
    /// The [`Url`] of a [`Stream`]
    pub fn url(&self) -> &str {
        &self.format.url
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
                .call()?;

            let content_length = res
                .header("Content-Length")
                .expect("HEAD request did not have a content-length");

            Ok(u64::from_str(content_length)
                .expect("Invalid content-length in HEAD response"))
        }
    }

    /// Get the [`Stream`] as a [`AsyncStream`](futures_core::Stream) of [`Bytes`](bytes::Bytes)
    pub async fn get(
        &self,
    ) -> Result<impl Read, ureq::Error> {
        Ok(self
            .client
            .api
            .http
            .get(self.url())
            .call()?
            .into_reader())
    }

    /// The [mime type](https://en.wikipedia.org/wiki/Media_type) of a [`Stream`]
    pub fn mime_type(&self) -> &str {
        &self.format.mime_type
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
