use std::collections::HashMap;

use once_cell::sync::Lazy;
use reqwest::IntoUrl;
use serde::Serialize;

use crate::youtube::{player_response::PlayerResponse, ytcfg::YtCfg};

static BASE_URL: &str = "https://youtubei.googleapis.com/youtubei/v1";

static CONTEXT: Lazy<serde_json::Value> = Lazy::new(|| {
    serde_json::json!({
        "client": {
          "hl": "en",
          "gl": "US",
          "clientName": "WEB",
          "clientVersion": "2.20210622.10.00"
        }
    })
});

pub enum ChannelPage {
    About,
    Videos,
    Playlists,
    Channels,
    Community,
}

pub enum Browse {
    Playlist(crate::playlist::Id),
    Channel {
        id: crate::channel::Id,
        page: ChannelPage,
    },
}

#[derive(Debug)]
pub struct Api {
    ytcfg: YtCfg,
    http: reqwest::Client,
}

impl Api {
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

        let request = Request {
            context: &CONTEXT,
            video_id: id,
        };

        self.get(format!("{}/player", BASE_URL), request).await
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

        let request = Request {
            context: &CONTEXT,
            video_id: id,
        };

        self.get(format!("{}/next", BASE_URL), request).await
    }

    pub async fn browse<T: serde::de::DeserializeOwned>(&self, browse: Browse) -> crate::Result<T> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            browse_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<String>,
        }

        let request = match browse {
            Browse::Playlist(id) => Request {
                context: &CONTEXT,
                browse_id: format!("VL{}", id),
                params: Some(base64::encode([0xc2, 0x06, 0x02, 0x08, 0x00])),
            },
            Browse::Channel { id, page } => Request {
                context: &CONTEXT,
                browse_id: format!("{}", id),
                params: match page {
                    ChannelPage::About => Some(base64::encode(b"about")),
                    ChannelPage::Videos => Some(base64::encode(b"videos")),
                    ChannelPage::Playlists => Some(base64::encode(b"playlists")),
                    ChannelPage::Channels => Some(base64::encode(b"channels")),
                    ChannelPage::Community => Some(base64::encode(b"community")),
                },
            },
        };

        self.get(format!("{}/browse", BASE_URL), request).await
    }

    pub async fn continuation<T: serde::de::DeserializeOwned>(
        &self,
        continuation: impl AsRef<str>,
    ) -> crate::Result<T> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            continuation: &'a str,
        }

        let request = Request {
            context: &CONTEXT,
            continuation: continuation.as_ref(),
        };

        self.get(format!("{}/browse", BASE_URL), request).await
    }

    pub async fn get_video_info(&self, id: crate::video::Id) -> crate::Result<PlayerResponse> {
        static URL: &str = "https://youtube.com/get_video_info";

        #[serde_with::serde_as]
        #[derive(Serialize, Debug)]
        struct Query<'a> {
            #[serde_as(as = "serde_with::DisplayFromStr")]
            video_id: crate::video::Id,
            el: &'a str,
            eurl: String,
            hl: &'a str,
            html5: usize,
            c: &'a str,
            cver: &'a str,
        }

        let query = Query {
            video_id: id,
            el: "embedded",
            eurl: format!("https://youtube.googleapis.com/v/{}", id),
            hl: "en",
            html5: 1,
            c: "TVHTML5",
            cver: "6.20180913",
        };

        let response = self
            .http
            .get(URL)
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        Ok(serde_json::from_str(
            &serde_urlencoded::from_str::<HashMap<String, String>>(&response)
                .expect("VideoInfo response was invalid urlencoded")["player_response"],
        )
        .expect("get_video_info player_response was invalid json"))
    }
}
