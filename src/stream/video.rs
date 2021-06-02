use crate::youtube::player_response::VideoFormat;

/// A [`Stream`](super::Stream) specifically containing video data.
#[derive(Debug)]
pub struct Stream {
    pub(super) common: super::common::Stream,
    pub(super) video: VideoFormat,
}

impl std::ops::Deref for Stream {
    type Target = super::common::Stream;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Stream {
    /// The width of a [`Stream`]
    pub fn width(&self) -> u64 {
        self.video.width
    }

    /// The height of a [`Stream`]
    pub fn height(&self) -> u64 {
        self.video.height
    }

    /// The frames per second of a [`Stream`]
    pub fn fps(&self) -> u64 {
        self.video.fps
    }
}
