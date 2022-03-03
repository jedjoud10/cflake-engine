use std::io::Cursor;

use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
#[derive(Default)]
pub struct AudioSource {
    // The source's ID in the playback
    pub(crate) idx: Option<usize>,

    // A temporary location for our loaded rodio decoder before we add it to the playback cache
    pub(crate) temp: Option<Buffered<Decoder<Cursor<Vec<u8>>>>>,
}

// Each audio source is loadable
impl assets::Asset for AudioSource {
    fn load_raw(_meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<AudioSource>
    where
        AudioSource: Sized {
        // Rodio moment
        let cursor = Cursor::new(bytes.to_vec());
        let read = Decoder::new(cursor).unwrap().buffered();
        Some(AudioSource {
            idx: None,
            temp: Some(read),
        })
    }
}
