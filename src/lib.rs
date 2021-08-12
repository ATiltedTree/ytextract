#![doc = include_str!("../README.md")]
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
