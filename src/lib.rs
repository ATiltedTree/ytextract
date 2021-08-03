//! A Library for extracting information from YouTube pages.
//!
//! # Notes
//!
//! ##### Subscriber count
//!
//! All functions that return subscriber counts only return 3-digit precision
//! values as that is all that YouTube returns. That means if channel has
//! exactly `164_583` subscribers, this library will return `164_000`.
//!
//! ##### Panic behavior
//!
//! This library should never panic. If it does, it should be reported as a
//! bug. Panics mostly mean, that YouTube changed something that this library
//! could not deal with.
//!
//! # Basic Example
//!
//! ```rust
//! # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Get a Client for making request
//! let client = ytextract::Client::new().await?;
//!
//! // Get information about the Video identified by the id "nI2e-J6fsuk".
//! let video = client.video("nI2e-J6fsuk".parse()?).await?;
//!
//! // Print the title of the Video
//! println!("Title: {}", video.title());
//! # Ok(())
//! # }
//! ```

#![deny(
    missing_docs,
    unsafe_code,
    missing_debug_implementations,
    rust_2018_idioms
)]

#[macro_use]
pub(crate) mod id;

pub mod channel;
mod client;
pub mod error;
pub(crate) mod player;
pub mod playlist;
pub mod stream;
mod thumbnail;
pub mod video;
pub(crate) mod youtube;

pub use channel::Channel;
pub use client::Client;
pub use error::Error;
pub use playlist::Playlist;
pub use stream::Stream;
pub use thumbnail::Thumbnail;
pub use video::Video;

/// The Result type used by this library
pub type Result<T> = std::result::Result<T, Error>;
