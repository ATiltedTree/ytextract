use crate::video::{self, Video};

/// A Client capable of interacting with YouTube
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    /// Create a new [`Client`]
    ///
    /// # Errors
    ///
    /// None. Currently.
    pub async fn new() -> crate::Result<Self> {
        Ok(Client {
            client: reqwest::Client::new(),
        })
    }

    /// Get a [`Video`] identified by its [`Id`][video::Id]
    ///
    /// # Errors
    ///
    /// [`Error::MissingData`]: If important data was not found
    /// [`Error::JSON`]: If json was unable to be parsed not found
    ///
    /// [Error]: crate::Error
    pub async fn video(&self, id: video::Id) -> crate::Result<Video> {
        Video::get(self.client.clone(), id).await
    }
}
