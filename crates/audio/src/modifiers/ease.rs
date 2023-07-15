use crate::Source;


// Changes how an audio source sounds like over time
pub enum Easing {
    // Sine easing
    Sine,

    // Linear easing
    Linear,

    // Exponential easing based on base factor
    Expo(f32),
}

// Easing direction
pub trait EasingDirection {}
pub struct FadeIn;
pub struct FadeOut;
impl EasingDirection for FadeIn {}
impl EasingDirection for FadeOut {}

// Fade using a specific easing function in a specific direction
pub struct Fade<T: Source, D: EasingDirection>(T, D, Easing);