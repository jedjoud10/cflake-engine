use thiserror::Error;

// An error that might occur during clip deserialization
#[derive(Debug, Error)]
pub enum AudioClipDeserializationError {
    #[error("MiniMP3 deserialization error {0}")]
    MP3(minimp3::Error),

    #[error("Wav IO error {0}")]
    Wav(std::io::Error),

    #[error("The given bit-depth of the audio clip is not supported")]
    BitDepthNotSupported,
}
