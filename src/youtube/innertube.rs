use once_cell::sync::Lazy;
use reqwest::{IntoUrl, Url};
use serde::Serialize;

use crate::youtube::{player_response::PlayerResponse, ytcfg::YtCfg};

static BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://www.youtube.com/youtubei/v1/").unwrap());

#[derive(Debug)]
pub struct API {
    ytcfg: YtCfg,
    http: reqwest::Client,
}

impl API {
    pub fn new(http: reqwest::Client, ytcfg: YtCfg) -> Self {
        Self { ytcfg, http }
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        request: impl serde::Serialize,
    ) -> crate::Result<T> {
        Ok(self
            .http
            .post(url)
            .query(&[("key", &self.ytcfg.innertube_api_key)])
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn player(&self, id: crate::video::Id) -> crate::Result<PlayerResponse> {
        #[serde_with::serde_as]
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            #[serde_as(as = "serde_with::DisplayFromStr")]
            video_id: crate::video::Id,
        }

        static URL: Lazy<Url> = Lazy::new(|| BASE_URL.join("player").unwrap());

        let request = Request {
            context: &self.ytcfg.innertube_context,
            video_id: id,
        };
        self.get(URL.clone(), request).await
    }

    pub async fn next(&self, id: crate::video::Id) -> crate::Result<serde_json::Value> {
        #[serde_with::serde_as]
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            #[serde_as(as = "serde_with::DisplayFromStr")]
            video_id: crate::video::Id,
        }

        static URL: Lazy<Url> = Lazy::new(|| BASE_URL.join("next").unwrap());

        let request = Request {
            context: &self.ytcfg.innertube_context,
            video_id: id,
        };
        self.get(URL.clone(), request).await
    }
}
