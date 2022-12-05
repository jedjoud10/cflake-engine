use crate::{PlayableAudioSamples, AudioClipError, AudioSamplesDescriptor};
use assets::Asset;
use std::{
    io::{BufReader, Cursor},
    sync::Arc,
};

// This is an audio clip that we can import from an mp3/wav file
// Audio clips must be clonable since we should be able to clone them to reuse them instead of loading new ones every time
#[derive(Clone)]
pub struct AudioClip(pub(crate) Arc<dyn PlayableAudioSamples>);

impl Asset for AudioClip {
    type Args<'args> = ();
    type Err = AudioClipError;

    fn extensions() -> &'static [&'static str] {
        &["mp3", "wav"]
    }

    fn deserialize(
        data: assets::Data,
        _args: Self::Args<'_>,
    ) -> Result<Self, Self::Err> {
        let samples = match data.extension() {
            // Decode an MP3 file into the appropriate format
            "mp3" => {
                let mut decoded = minimp3::Decoder::new(data.bytes());
                let mut frames = Vec::<minimp3::Frame>::new();

                // Handle decoding a singular frame
                fn decode(result: Result<minimp3::Frame, minimp3::Error>) -> Result<Option<minimp3::Frame>, AudioClipError> {
                    match result {
                        Ok(frame) => Ok(Some(frame)),
                        Err(minimp3::Error::Eof) => Ok(None),
                        Err(err) => Err(AudioClipError::MP3(err)),
                    }
                }

                // Load the frames in, and return any errors (other than EoF)
                while let Some(frame) = decode(decoded.next_frame())? {
                    frames.push(frame);
                }
                log::debug!("Loaded {} frames from the MP3 file {:?}", frames.len(), data.path());

                // Create a samples descriptor from the first frame
                let descriptor = AudioSamplesDescriptor {
                    bitrate: frames[0].bitrate as u32,
                    sample_rate: frames[0].sample_rate as u32,
                    channels: frames[0].channels as u16,
                };

                // Convert to samples
                let samples = frames.into_iter().flat_map(|frame| {
                    frame.data.into_iter()
                }).collect::<Vec<i16>>();
                let arc: Arc<[i16]> = samples.into();

                // Create the samples trait object
                let to: Arc<dyn PlayableAudioSamples> = Arc::new((arc, descriptor));
                to                
            }

            // Decode a WAV file into the appropriate format
            "wav" => {
                let mut read =
                    BufReader::new(Cursor::new(data.bytes()));
                let (header, data) = wav::read(&mut read)
                    .map_err(AudioClipError::Wav)?;

                // Create a samples descriptor
                let descriptor = AudioSamplesDescriptor {
                    bitrate: header.bytes_per_second / 1000,
                    sample_rate: header.sampling_rate,
                    channels: header.channel_count,
                };

                todo!()
            }
            _ => panic!(),
        };

        Ok(Self(samples))
    }
}
