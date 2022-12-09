use cpal::Sample;

// An audio output is ANYTHING that makes sound
pub trait AudioOutput<T: Sample> {}

// Audio generators create new sound by reading from a file or creating it using a wave type
pub trait AudioGenerator<T: Sample>: AudioOutput<T> {}

// Audio modifiers change how audio outputs sound
pub trait AudioModifier<T: Sample>: AudioOutput<T> {}

// Audio mixers simply combine 2 or more audio outputs together
pub trait AudioMixer<T: Sample>: AudioOutput<T> {}