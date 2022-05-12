use std::io::Cursor;

use assets::Asset;
use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
pub struct AudioSource {
    pub(crate) buffered: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl Asset<'static> for AudioSource {
    type OptArgs = ();

    fn is_extension_valid(extension: &str) -> bool {
        match extension {
            "ogg" => true,
            "mp3" => true,
            "wav" => true,
            _ => false,
        } 
    }

    fn deserialize(bytes: &[u8], args: Self::OptArgs) -> Self {
        let cursor = Cursor::new(bytes.to_vec());
        let read = Decoder::new(cursor).ok().unwrap().buffered();
        AudioSource { buffered: read }    
    }    
}
