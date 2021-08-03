use crate::youtube::player_response::VideoFormat;

/// A [`Stream`](super::Stream) specifically containing video data.
#[derive(Clone)]
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

    pub(super) fn debug(&self, debug: &mut std::fmt::DebugStruct<'_, '_>) {
        debug
            .field("width", &self.width())
            .field("height", &self.height())
            .field("fps", &self.fps());
    }
}

impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("VideoStream");
        self.debug(&mut debug);
        debug.finish()
    }
}
