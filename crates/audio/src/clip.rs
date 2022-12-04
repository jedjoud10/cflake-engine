use crate::AudioSamples;
use assets::Asset;
use std::{
    io::{BufReader, Cursor},
    sync::Arc,
};

// This is an audio clip that we can import from an mp3/wav file
// Audio clips must be clonable since we use them a jack shit ton
#[derive(Clone)]
pub struct AudioClip(Arc<dyn AudioSamples>);

impl Asset for AudioClip {
    type Args<'args> = ();

    fn extensions() -> &'static [&'static str] {
        &["mp3", "wav"]
    }

    fn deserialize(
        data: assets::Data,
        _args: Self::Args<'_>,
    ) -> Self {
        match data.extension() {
            // Decode an MP3 file into the appropriate format
            "mp3" => {
                let mut decoded = minimp3::Decoder::new(data.bytes());
                let mut frames = Vec::<minimp3::Frame>::new();

                // Load the frames in
                while let Ok(frame) = decoded.next_frame() {
                    frames.push(frame);
                }

                let first = decoded.next_frame().unwrap();
                let minimp3::Frame {
                    data: _,
                    sample_rate: _,
                    channels: _,
                    layer: _,
                    bitrate: _,
                } = first;
            }

            // Decode a WAV file into the appropriate format
            "wav" => {
                let mut read =
                    BufReader::new(Cursor::new(data.bytes()));
                let (_header, _data) = wav::read(&mut read).unwrap();
            }
            _ => panic!(),
        }

        todo!()
    }
}
