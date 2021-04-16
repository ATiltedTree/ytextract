//! A Library for extracting information from YouTube pages.
//!
//! # Basic Example
//!
//! ```rust
//! # #[async_std::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Get a Client for making request
//! let client = youtube_extractor::Client::new().await?;
//!
//! // Get information about the Video identified by the id "nI2e-J6fsuk".
//! let video = client.video("nI2e-J6fsuk".parse()?).await?;
//!
//! // Print the title of the Video
//! println!("Title: {}", video.title());
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs, rustdoc::missing_crate_level_docs, unsafe_code)]

pub mod channel;
mod client;
mod error;
mod id;
mod thumbnail;
pub(crate) mod utils;
pub mod video;

pub use client::Client;
pub use error::Error;
pub use id::Id;
pub use thumbnail::Thumbnail;
pub use video::Video;

/// The Result type used by this library
pub type Result<T> = std::result::Result<T, Error>;
