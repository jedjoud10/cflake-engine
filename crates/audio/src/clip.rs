use std::{any::Any, sync::Arc, io::{BufReader, Cursor}};
use assets::Asset;
use crate::AudioSamples;

// This is a sound clip that we can import from an mp3/wav file
pub struct SoundClip(Arc<dyn AudioSamples>);

impl Asset<'static> for SoundClip {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["mp3", "wav"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        match data.extension() {
            // Decode an MP3 file into the appropriate format
            "mp3" => {
                let mut decoded = minimp3::Decoder::new(data.bytes());
                let first = decoded.next_frame().unwrap();
                let minimp3::Frame { data, sample_rate, channels, layer, bitrate } = first;
            },

            // Decode a WAV file into the appropriate format
            "wav" => {
                let mut read = BufReader::new(Cursor::new(data.bytes()));
                let (header, data) = wav::read(&mut read).unwrap();
            },
            _ => panic!()
        }

        todo!()
    }
}