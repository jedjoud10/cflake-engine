use std::io::Cursor;

use assets::{Asset, loader::AssetBytes};
use rodio::{source::Buffered, Decoder, Source};

// A single audio source that can be loaded
pub struct AudioSource {
    pub(crate) buffered: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl Asset<'static> for AudioSource {
    type Args = ();

    fn is_extension_valid(extension: &str) -> bool {
        match extension {
            "ogg" | "mp3" | "wav" => true,
            _ => false,
        } 
    }

    fn deserialize(bytes: AssetBytes, args: Self::Args) -> Self {
        let cursor = Cursor::new(bytes.as_ref().to_vec());
        let read = Decoder::new(cursor).ok().unwrap().buffered();
        AudioSource { buffered: read }    
    }    
}
