use std::io::Cursor;
use assets::Asset;
use rodio::{source::Buffered, Decoder, Source};
type Data = Buffered<Decoder<Cursor<Vec<u8>>>>;

// A single audio clip that will be loaded from .mp3, .ogg, or .wav files
pub struct AudioClip(pub(crate) Data);

impl Asset<'static> for AudioClip {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["mp3", "ogg", "wav"]
    }

    fn deserialize(data: assets::Data, _args: Self::Args) -> Self {
        let cursor = Cursor::new(data.bytes().to_vec());
        let read = Decoder::new(cursor).ok().unwrap().buffered();
        AudioClip(read)
    }
}
