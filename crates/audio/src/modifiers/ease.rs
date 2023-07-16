use std::time::Duration;

use crate::Source;


// Changes how an audio source sounds like over time
pub enum Easing {
    // Sine easing
    Sine,

    // Cosine easing
    Cosine, 

    // Linear easing
    Linear,

    // Exponential easing based on base factor
    Expo(f32)
}

// Easing direction
pub enum EasingDirection {
    In,
    Out,
}

// Fade using a specific easing function in a specific direction
pub struct Fade<T: Source>(pub(crate) T, pub(crate) Easing, pub(crate) EasingDirection, pub(crate) Duration);

impl<T: Source> Source for Fade<T> {
    fn cache(&mut self) {
        self.0.cache();
    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        let percent = input.duration / self.3.as_secs_f32();

        if percent > 1.0 {
            match self.2 {
                EasingDirection::In => return self.0.sample(input),
                EasingDirection::Out => return None,
            }
        };
        
        let mult = match (&self.1, &self.2) {
            // Sine easing function (linear -> ease)
            (Easing::Sine, dir) => {
                let eased = (percent * std::f32::consts::PI / 2.0).sin();

                match dir {
                    EasingDirection::In => eased,
                    EasingDirection::Out => 1.0 - eased,
                }
            },

            // Cosine easing function (ease -> linear)
            (Easing::Cosine, dir) => {
                let eased = 1.0 - (percent * std::f32::consts::PI / 2.0).cos();

                match dir {
                    EasingDirection::In => eased,
                    EasingDirection::Out => 1.0 - eased,
                }
            },
            
            _ => todo!(),
        };

        self.0.sample(input).map(|x| x * mult)
    }

    fn duration(&self) -> Option<std::time::Duration> {
        None
    }

    fn target_channels(&self) -> Option<u16> {
        self.0.target_channels()
    }

    fn target_sample_rate(&self) -> Option<u32> {
        self.0.target_sample_rate()
    }
}