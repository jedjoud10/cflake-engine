use std::io::Cursor;

use assets::{Asset};
use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
pub struct AudioSource {
    pub(crate) buffered: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl Asset<'static> for AudioSource {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["mp3", "ogg", "wav"]
    }
    
    fn deserialize(bytes: &[u8], args: Self::Args, meta: assets::loader::Meta) -> Self {
        let cursor = Cursor::new(bytes.as_ref().to_vec());
        let read = Decoder::new(cursor).ok().unwrap().buffered();
        AudioSource { buffered: read }
    }
}
