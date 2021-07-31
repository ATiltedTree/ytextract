use once_cell::sync::Lazy;
use reqwest::IntoUrl;
use serde::Serialize;

use crate::youtube::{
    player_response,
    tv_config::{Config, TvConfig},
};

const BASE_URL: &str = "https://youtubei.googleapis.com/youtubei/v1";

const TV_CONFIG_URL: &str = "https://www.youtube.com/tv_config?action_get_config=true";

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

static EMBEDDED_CONTEXT: Lazy<serde_json::Value> = Lazy::new(|| {
    serde_json::json!({
        "client": {
            "hl": "en",
            "gl": "US",
            "clientName": "WEB",
            "clientScreen":"EMBED",
            "clientVersion": "1.20210620.0.1"
        }
    })
});

pub enum ChannelPage {
    About,
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
    pub(crate) config: Config,
    http: reqwest::Client,
}

impl Api {
    pub async fn new(http: reqwest::Client) -> crate::Result<Self> {
        let tv_config = http
            .get(TV_CONFIG_URL)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let tv_config = tv_config
            .lines()
            .nth(1)
            .expect("tv_config did not have a second line");

        let tv_config: TvConfig =
            serde_json::from_str(tv_config).expect("tv_config was invalid json");

        Ok(Self {
            config: tv_config
                .web_player_context_config
                .web_player_context_config_id_living_room_watch,
            http,
        })
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: impl IntoUrl,
        request: impl serde::Serialize,
    ) -> crate::Result<T> {
        Ok(self
            .http
            .post(url)
            .query(&[("key", &self.config.innertube_api_key)])
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn streams(
        &self,
        id: crate::video::Id,
    ) -> crate::Result<player_response::StreamResult> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            video_id: crate::video::Id,
        }

        let request = Request {
            context: &CONTEXT,
            video_id: id,
        };

        let res: player_response::StreamResult =
            self.get(format!("{}/player", BASE_URL), request).await?;

        match res.into_std() {
            Ok(streaming_data) => Ok(player_response::StreamResult::Ok { streaming_data }),
            Err(crate::Error::Youtube(err)) if err.is_streamable() => {
                let request = Request {
                    context: &EMBEDDED_CONTEXT,
                    video_id: id,
                };
                self.get(format!("{}/player", BASE_URL), request).await
            }
            Err(err) => Err(err),
        }
    }

    pub async fn player(&self, id: crate::video::Id) -> crate::Result<player_response::Result> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
            video_id: crate::video::Id,
        }

        let request = Request {
            context: &CONTEXT,
            video_id: id,
        };

        self.get(format!("{}/player", BASE_URL), request).await
    }

    pub async fn next(&self, id: crate::video::Id) -> crate::Result<serde_json::Value> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            context: &'a serde_json::Value,
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
                    ChannelPage::About => Some(base64::encode(b"\x12\x05about")),
                    ChannelPage::Playlists => Some(base64::encode(b"\x12\x09playlists")),
                    ChannelPage::Channels => Some(base64::encode(b"\x12\x08channels")),
                    ChannelPage::Community => Some(base64::encode(b"\x12\x09community")),
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
}
