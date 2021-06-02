use crate::{
    youtube::player_response::{CommonFormat, Quality},
    Client,
};

use chrono::{DateTime, Utc};
use reqwest::Url;

use std::{collections::HashMap, sync::Arc, time::Duration};

/// A [`Stream`](super::Stream) containing video or audio data.
#[derive(Debug)]
pub struct Stream {
    pub(super) format: CommonFormat,
    pub(super) client: Arc<Client>,
}

impl Stream {
    /// The [`Url`] of a [`Stream`]
    pub async fn url(&self) -> crate::Result<Url> {
        match &self.format.url {
            Some(url) => Ok(url.clone()),
            None => {
                let signature_cipher = self
                    .format
                    .signature_cipher
                    .as_ref()
                    .expect("Stream did not have a URL or signatureCipher");
                let root: HashMap<String, String> =
                    serde_urlencoded::from_str(signature_cipher.as_str())
                        .expect("signatureCipher was not urlencoded");

                let signature = self.client.player().await.cipher().run(root["s"].clone());
                let signature_arg = &root["sp"];
                let mut url = Url::parse(&root["url"])
                    .expect("signatureCipher url attribute was not a valid URL");

                let query = url
                    .query()
                    .map(|q| format!("{}&{}={}", q, signature_arg, signature));

                if let Some(query) = query {
                    url.set_query(Some(&query));
                } else {
                    panic!(
                        "URL('{}') did not have a query while trying to add '{}={}'",
                        url, signature_arg, signature
                    );
                }
                Ok(url)
            }
        }
    }

    /// The length of a [`Stream`] in bytes
    pub async fn content_length(&self) -> crate::Result<u64> {
        if let Some(content_length) = self.format.content_length {
            Ok(content_length)
        } else {
            let res = self
                .client
                .client
                .head(self.url().await?)
                .send()
                .await?
                .error_for_status()?;

            Ok(res
                .content_length()
                .ok_or(super::Error::UnknownContentLength)?)
        }
    }

    /// Get the [`Stream`] as a [`AsyncStream`](futures_core::Stream) of [`Bytes`](bytes::Bytes)
    pub async fn get(
        &self,
    ) -> crate::Result<impl futures_core::Stream<Item = Result<bytes::Bytes, reqwest::Error>>> {
        Ok(self
            .client
            .client
            .get(self.url().await?)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream())
    }

    /// The [`Quality`] of a [`Stream`]
    pub fn quality(&self) -> &Quality {
        &self.format.quality
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
}
