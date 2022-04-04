use std::time::Duration;

use serde::Serialize;

use crate::{youtube::player_response, Error};

const RETRYS: u32 = 5;
const TIMEOUT: Duration = Duration::from_secs(30);
const DUMP: bool = option_env!("YTEXTRACT_DUMP").is_some();
const BASE_URL: &str = "https://youtubei.googleapis.com/youtubei/v1";
const API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

const CONTEXT_WEB: Context<'static> = Context {
    client: Client {
        hl: "en",
        gl: "US",
        client_name: "WEB",
        client_version: "2.20210622.10.0",
    },
};

const CONTEXT_ANDROID: Context<'static> = Context {
    client: Client {
        hl: "en",
        gl: "US",
        client_name: "ANDROID",
        client_version: "16.05",
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Context<'a> {
    client: Client<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Client<'a> {
    hl: &'a str,
    gl: &'a str,
    client_name: &'a str,
    client_version: &'a str,
}

pub enum ChannelPage {
    About,
}

pub enum Browse {
    Playlist(crate::playlist::Id),
    Channel {
        id: crate::channel::Id,
        page: ChannelPage,
    },
    Continuation(String),
}

pub enum Next {
    Video(crate::video::Id),
    Continuation(String),
}

#[derive(Clone)]
pub struct Api {
    pub(crate) http: ureq::Agent,
}

fn dump(endpoint: &'static str, response: &str) {
    let _ = std::fs::create_dir(endpoint);
    use std::time::SystemTime;
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("TIME");
    std::fs::write(
        &format!("{}/{}.json", endpoint, time.as_millis()),
        &response,
    )
    .expect("Write");
}

impl Default for Api {
    fn default() -> Self {
        Self {
            http: ureq::AgentBuilder::new()
                .timeout(TIMEOUT)
                .build()
        }
    }
}

impl Api {
    async fn get<T: serde::de::DeserializeOwned, R: Serialize + Send + Sync>(
        &self,
        endpoint: &'static str,
        request: R,
        context: Context<'static>,
    ) -> crate::Result<T> {
        #[derive(Serialize)]
        struct Request<R: Serialize> {
            context: Context<'static>,
            #[serde(flatten)]
            request: R,
        }

        let request = Request { context, request };

        let http_req = self
            .http
            .post(&format!("{}/{}", BASE_URL, endpoint))
            .set("X-Goog-Api-Key", API_KEY);

        let mut retry = 0;

        loop {
            let response = http_req
                .clone()
                .send_json(&request);

            match response {
                Ok(res) => {
                    // following unwrap will fail on large (> 10MB) responses
                    let res = res.into_string().unwrap();
                    if DUMP {
                        dump(endpoint, &res)
                    }
                    break Ok(serde_json::from_str::<T>(&res).expect("Failed to parse JSON"));
                }
                Err(err) => {
                    if retry == RETRYS {
                        log::error!("Timed out {} times. Stopping...", RETRYS);
                        break Err(Error::Request(err));
                    } else {
                        log::warn!("Timeout reached, retrying...");
                        retry += 1;
                        continue;
                    }
                }
            }
        }
    }

    pub async fn streams(
        &self,
        id: crate::video::Id,
    ) -> crate::Result<player_response::Result<player_response::StreamPlayerResponse>> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            video_id: crate::video::Id,
        }

        let request = Request { video_id: id };

        self.get("player", request, CONTEXT_ANDROID).await
    }

    pub async fn player(
        &self,
        id: crate::video::Id,
    ) -> crate::Result<player_response::Result<player_response::PlayerResponse>> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            video_id: crate::video::Id,
        }

        let request = Request { video_id: id };

        self.get("player", request, CONTEXT_ANDROID).await
    }

    pub async fn next<T: serde::de::DeserializeOwned>(&self, next: Next) -> crate::Result<T> {
        match next {
            Next::Video(video_id) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    video_id: crate::video::Id,
                }

                let request = Request { video_id };

                self.get("next", request, CONTEXT_WEB).await
            }
            Next::Continuation(continuation) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    continuation: String,
                }

                let request = Request { continuation };

                self.get("next", request, CONTEXT_WEB).await
            }
        }
    }

    pub async fn browse<T: serde::de::DeserializeOwned>(&self, browse: Browse) -> crate::Result<T> {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            browse_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<String>,
        }

        let request = match browse {
            Browse::Playlist(id) => Request {
                browse_id: format!("VL{}", id),
                params: Some(base64::encode([0xc2, 0x06, 0x02, 0x08, 0x00])),
            },
            Browse::Channel { id, page } => Request {
                browse_id: format!("{}", id),
                params: match page {
                    ChannelPage::About => Some(base64::encode(b"\x12\x05about")),
                },
            },
            Browse::Continuation(continuation) => {
                #[derive(Debug, Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Request {
                    continuation: String,
                }

                let request = Request { continuation };

                return self.get("browse", request, CONTEXT_WEB).await;
            }
        };

        self.get("browse", request, CONTEXT_WEB).await
    }
}
