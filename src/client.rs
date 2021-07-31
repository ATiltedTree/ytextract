use once_cell::sync::OnceCell;

use crate::{
    channel, player::Player, playlist, stream, video, youtube::innertube::Api, Channel, Playlist,
    Stream, Video,
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
        Ok(Arc::new(Self {
            player: OnceCell::new(),
            api: Api::new(http.clone()).await?,
            http,
        }))
    }

    pub(crate) async fn init_player(&self) {
        if self.player.get().is_none() {
            let player = Player::from_url(&self.http, self.api.config.js_url())
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

    /// Get a [`Channel`] identified by a [`Id`](channel::Id)
    pub async fn channel(self: &Arc<Self>, id: channel::Id) -> crate::Result<Channel> {
        Channel::get(Arc::clone(self), id).await
    }
}
