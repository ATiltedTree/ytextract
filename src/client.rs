use crate::{
    player::Player,
    stream,
    video::{self, Video},
    youtube::{innertube::API, ytcfg::YtCfg},
    Stream,
};

use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;

use std::sync::Arc;

static YTCFG_EXP: Lazy<Regex> = Lazy::new(|| Regex::new(r"\nytcfg.set\((\{.*\})\);").unwrap());

// Common UA. Hide ourself in the UserAgent mud.
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.131 Safari/537.36";

static DEFAULT_HEADERS: Lazy<reqwest::header::HeaderMap> = Lazy::new(|| {
    use reqwest::header::*;
    let mut map = HeaderMap::new();

    macro_rules! header {
        ($key:expr, $val:literal) => {
            map.insert($key, reqwest::header::HeaderValue::from_static($val));
        };
    }

    // Bypass cookie consent
    header!(COOKIE, "CONSENT=YES+cb");
    // Make sure we get English
    header!(ACCEPT_LANGUAGE, "en-US,en;q=0.9");
    // HTTPS is always good
    header!(UPGRADE_INSECURE_REQUESTS, "1");

    map
});

/// A Client capable of interacting with YouTube
#[derive(Debug)]
pub struct Client {
    pub(crate) client: reqwest::Client,
    player: OnceCell<Player>,
    pub(crate) api: API,
    pub(crate) ytcfg: YtCfg,
}

impl Client {
    /// Create a new [`Client`]
    pub async fn new() -> crate::Result<Arc<Self>> {
        let http = reqwest::ClientBuilder::new()
            .default_headers(DEFAULT_HEADERS.clone())
            .user_agent(DEFAULT_USER_AGENT)
            .build()?;
        let home = http
            .get("https://youtube.com/?hl=en")
            .send()
            .await?
            .error_for_status()?;

        let body = home.text().await?;

        let ytcfg = YTCFG_EXP
            .captures(&body)
            .and_then(|c| c.get(1))
            .expect("YouTubeConfig was unable to be found. This is unrecoverable and should be reported.\nError")
            .as_str();
        let ytcfg: YtCfg = serde_json::from_str(ytcfg).expect("YoutubeConfig was not valid json");

        Ok(Arc::new(Client {
            player: OnceCell::new(),
            api: API::new(http.clone(), ytcfg.clone()),
            ytcfg,
            client: http,
        }))
    }

    pub(crate) async fn player(&self) -> &Player {
        if self.player.get().is_none() {
            let player = match Player::from_url(&self.client, &self.ytcfg.player_js_url).await {
                Ok(player) => player,
                Err(err) => {
                    panic!("\nA Error occurred while parsing a Player. This is unrecoverable and should be reported.\nError: {}\n", err)
                }
            };
            self.player.set(player).unwrap();
        }
        self.player.get().unwrap()
    }

    /// Get a [`Video`] identified by a [`Id`](video::Id)
    pub async fn video(self: &Arc<Self>, id: video::Id) -> crate::Result<Video> {
        Video::get(Arc::clone(self), id).await
    }

    /// Get the [`Stream`]s of a [`Video`] identified by a [`Id`](video::Id)
    pub async fn streams(
        self: &Arc<Self>,
        id: video::Id,
    ) -> crate::Result<impl Iterator<Item = Stream>> {
        stream::get(Arc::clone(self), id, None).await
    }
}
