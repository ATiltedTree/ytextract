use crate::youtube::player_response::AudioFormat;

/// A [`Stream`](super::Stream) specifically containing audio data.
pub struct Stream {
    pub(super) common: super::common::Stream,
    pub(super) audio: AudioFormat,
}

impl std::ops::Deref for Stream {
    type Target = super::common::Stream;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Stream {
    /// The sample rate of a [`Stream`]
    pub fn sample_rate(&self) -> u64 {
        self.audio.audio_sample_rate
    }

    /// The amount of channels of a [`Stream`]
    pub fn channels(&self) -> u64 {
        self.audio.audio_channels
    }

    pub(super) fn debug(&self, debug: &mut std::fmt::DebugStruct) {
        debug
            .field("sample_rate", &self.sample_rate())
            .field("channels", &self.channels());
    }
}

impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("AudioStream");
        self.debug(&mut debug);
        debug.finish()
    }
}
