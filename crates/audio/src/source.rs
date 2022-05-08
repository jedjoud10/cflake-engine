use std::io::Cursor;

use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
pub struct AudioSource {
    pub(crate) buffered: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl assets::Asset for AudioSource {
    type OptArgs = ();

    fn is_valid(meta: assets::metadata::AssetMetadata) -> bool {
        match meta.extension() {
            "ogg" => true,
            "mp3" => true,
            "wav" => true,
            _ => false,
        }
    }

    unsafe fn deserialize(bytes: &[u8], args: &Self::OptArgs) -> Option<Self> {
        let cursor = Cursor::new(bytes.to_vec());
        let read = Decoder::new(cursor).ok()?.buffered();
        Some(AudioSource { buffered: read })
    }
}
