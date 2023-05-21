use crate::AudioClipDeserializationError;
use assets::Asset;
use rayon::slice::ParallelSlice;
use std::{
    io::{BufReader, Cursor},
    sync::Arc,
    time::Duration,
};

// This is an audio clip that we can import from an mp3/wav file
// Audio clips must be clonable since we should be able to clone them to reuse them instead of loading new ones every time
// Audio clips always use f32 as their base sample format typ
#[derive(Clone)]
pub struct AudioClip {
    samples: Arc<[f32]>,
    bitrate: u32,
    sample_rate: u32,
    channels: u16,
    duration: Duration,
}

impl AudioClip {
    // Get the bitrate of the audio samples in kb/s
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }

    // Get the sample rate of the audio samples in hertz
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    // Get the number of channels in the audio samples
    pub fn channels(&self) -> u16 {
        self.channels
    }

    // Get the duration of the audio clip
    pub fn duration(&self) -> Duration {
        self.duration
    }

    // Get access to the internally stored samples
    // Multi-channel samples are stored like this:
    // 000000 111111 222222 333333
    // where each number represents the channel it's used for
    pub fn samples(&self) -> Arc<[f32]> {
        self.samples.clone()
    }
}

impl Asset for AudioClip {
    type Context<'ctx> = ();
    type Settings<'stg> = ();
    type Err = AudioClipDeserializationError;

    fn extensions() -> &'static [&'static str] {
        &["mp3", "wav"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        _: Self::Context<'c>,
        _: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        match data.extension() {
            // Decode an MP3 file into the appropriate format
            "mp3" => {
                let mut decoded = minimp3::Decoder::new(data.bytes());
                let mut frames = Vec::<minimp3::Frame>::new();

                // Handle decoding a singular frame
                fn decode(
                    result: Result<minimp3::Frame, minimp3::Error>,
                ) -> Result<Option<minimp3::Frame>, AudioClipDeserializationError> {
                    match result {
                        Ok(frame) => Ok(Some(frame)),
                        Err(minimp3::Error::Eof) => Ok(None),
                        Err(err) => Err(AudioClipDeserializationError::MP3(err)),
                    }
                }

                // Load the frames in, and return any errors (other than EoF)
                while let Some(frame) = decode(decoded.next_frame())? {
                    frames.push(frame);
                }

                // Fetch the descriptor data
                let bitrate = frames[0].bitrate as u32;
                let sample_rate = frames[0].sample_rate as u32;
                let channels = frames[0].channels as u16;

                // Calculate the duration of this clip
                let duration = calculate_clip_duration_secs_from_frames(
                    frames.len(),
                    channels,
                    sample_rate,
                    frames[0].data.len(),
                );
                log::debug!(
                    "Loaded {} seconds from MP3 file {:?}",
                    duration.as_secs(),
                    data.path()
                );

                // Sum up all the frame samples together
                let samples = frames
                    .into_iter()
                    .flat_map(|frame| frame.data.into_iter())
                    .map(|x| cpal::Sample::to_f32(&x))
                    .collect::<Vec<f32>>();

                Ok(Self {
                    samples: samples.into(),
                    bitrate,
                    sample_rate,
                    channels,
                    duration,
                })
            }

            // Decode a WAV file into the appropriate format
            "wav" => {
                let mut read = BufReader::new(Cursor::new(data.bytes()));
                let (header, bitdepth) =
                    wav::read(&mut read).map_err(AudioClipDeserializationError::Wav)?;

                // Fetch the descriptor data
                let bitrate = header.bytes_per_second * 8;
                let sample_rate = header.sampling_rate;
                let channels = header.channel_count;

                // Calculate the duration of the audio clip
                let duration = calculate_clip_duration_secs_from_size(
                    data.bytes().len(),
                    header.bytes_per_second as usize,
                );
                log::debug!(
                    "Loaded {} seconds from WAV file {:?}",
                    duration.as_secs(),
                    data.path()
                );

                // Convert the bitdepth data into
                let samples = match bitdepth {
                    wav::BitDepth::Sixteen(vec) => vec
                        .into_iter()
                        .map(|x| cpal::Sample::to_f32(&x))
                        .collect::<Vec<f32>>(),
                    wav::BitDepth::ThirtyTwoFloat(vec) => vec,
                    _ => return Err(AudioClipDeserializationError::BitDepthNotSupported),
                };

                Ok(Self {
                    samples: samples.into(),
                    bitrate,
                    sample_rate,
                    channels,
                    duration,
                })
            }
            _ => panic!(),
        }
    }
}

// Calculate the clip duration knowing the number of frames, channels and sample rate
// https://chunminchang.github.io/blog/post/estimation-of-mp3-duration
fn calculate_clip_duration_secs_from_frames(
    frames: usize,
    channels: u16,
    sample_rate: u32,
    samples_per_frame: usize,
) -> Duration {
    let samples_per_frame = samples_per_frame as f32 / channels as f32;
    let total_frames = frames as f32;
    let sample_rate = sample_rate as f32;
    Duration::from_secs(((samples_per_frame * total_frames) / sample_rate) as u64)
}

// Calculate the clip duration using the file size and bytes per second
fn calculate_clip_duration_secs_from_size(file_size: usize, bytes_per_second: usize) -> Duration {
    let file_size = file_size as f32;
    let bytes_per_second = bytes_per_second as f32;
    Duration::from_secs((file_size / bytes_per_second) as u64)
}
