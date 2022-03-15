use std::io::Cursor;

use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
#[derive(Default)]
pub struct AudioSource {
    // Loaded bytes
    pub(crate) buffered: Option<Buffered<Decoder<Cursor<Vec<u8>>>>>,
}

// Each audio source is loadable
impl assets::Asset for AudioSource {
    fn deserialize(self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        // Rodio moment
        let cursor = Cursor::new(bytes.to_vec());
        let read = Decoder::new(cursor).unwrap().buffered();
        Some(AudioSource { buffered: Some(read) })
    }
}
