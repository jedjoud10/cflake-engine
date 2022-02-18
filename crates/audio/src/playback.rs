use std::io::Cursor;
use rodio::{Source, OutputStreamHandle, OutputStream, Decoder};
use crate::source::AudioSource;
// A playback cache that contains all the loaded sources
pub struct Playback {
    stream_handle: OutputStreamHandle,
    cache: Vec<Vec<u8>>,
}

impl Default for Playback {
    fn default() -> Self {
        // Get the stream handle
        let (_stream, handle) = OutputStream::try_default().unwrap();
        Self {
            stream_handle: handle,
            cache: Default::default()
        }
    }
}

impl Playback {
    // Play a sound
    pub fn play(&self, source: &AudioSource) -> Option<()> {
        // Check if the index is gud
        let idx = source.idx?;
        // Decode then play the sound        
        let compressed = self.cache.get(idx)?.clone();
        let cursor = Cursor::new(compressed);
        let decoded = Decoder::new(cursor).ok()?;
        self.stream_handle.play_raw(decoded.convert_samples()).ok()?;
        Some(())
    }
    // Append a sound to the playback cache (not really; We are just stealing it's temporary bytes)
    pub fn append(&mut self, mut source: AudioSource) -> Option<AudioSource> {
        // Steal
        source.idx = Some(self.cache.len());
        let compressed = source.temp.take()?;
        self.cache.push(compressed);
        Some(source)
    }
}