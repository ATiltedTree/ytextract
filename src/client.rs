use once_cell::sync::OnceCell;

use crate::{
    player::Player,
    playlist, stream,
    video::{self, Video},
    youtube::{innertube::Api, ytcfg::YtCfg},
    Playlist, Stream,
};

use std::sync::Arc;

/// A Client capable of interacting with YouTube
#[derive(Debug)]
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) player: OnceCell<Player>,
    pub(crate) api: Api,
}

impl Client {
    /// Create a new [`Client`]
    pub async fn new() -> crate::Result<Arc<Self>> {
        let http = reqwest::Client::new();
        let body = http
            .get("https://youtube.com/?hl=en")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let (_, ytcfg) = lazy_regex::regex_captures!(r"\nytcfg.set\((\{.*\})\);", &body)
            .expect("YoutubeConfig was unable to be found");
        let ytcfg: YtCfg = serde_json::from_str(ytcfg).expect("YoutubeConfig was not valid json");

        Ok(Arc::new(Client {
            player: OnceCell::new(),
            api: Api::new(http.clone(), ytcfg),
            http,
        }))
    }

    pub(crate) async fn init_player(&self) {
        if self.player.get().is_none() {
            let player = Player::from_url(&self.http, &self.api.ytcfg.player_js_url)
                .await
                .expect("Unable to parse player");

            if self.player.set(player).is_err() {
                log::warn!("Reinitialized player. Oh well...");
            }
        }
    }

    pub(crate) fn player(&self) -> &Player {
        self.player.get().expect("Player not initialized!")
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

    /// Get a [`Playlist`] identified by a [`Id`](playlist::Id)
    pub async fn playlist(self: &Arc<Self>, id: playlist::Id) -> crate::Result<Playlist> {
        Playlist::get(Arc::clone(self), id).await
    }
}
