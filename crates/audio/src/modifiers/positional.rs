use crate::{Source, Value};

pub struct Positional<L: Value<vek::Vec3<f32>>, E: Value<vek::Vec3<f32>>, T: Source>(
    pub(crate) T,
    pub(crate) E::Storage,
    pub(crate) [L::Storage; 2],
    pub(crate) [f32; 2],
);

impl<L: Value<vek::Vec3<f32>>, E: Value<vek::Vec3<f32>>, T: Source> Source for Positional<L, E, T> {
    fn cache(&mut self) {
        self.0.cache();

        E::cache(&mut self.1);
        L::cache(&mut self.2[0]);
        L::cache(&mut self.2[1]);

        let emitter = E::fetch(&self.1);
        let left = L::fetch(&self.2[0]);
        let right = L::fetch(&self.2[1]);

        // https://github.com/RustAudio/rodio/blob/master/src/source/spatial.rs
        let left_dist_sq = left.distance_squared(emitter);
        let right_dist_sq = right.distance_squared(emitter);
        let max_diff = left.distance(right);
        let left_dist = left_dist_sq.sqrt();
        let right_dist = right_dist_sq.sqrt();
        let left_diff_modifier = (((left_dist - right_dist) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
        let right_diff_modifier =
            (((right_dist - left_dist) / max_diff + 1.0) / 4.0 + 0.5).min(1.0);
        let left_dist_modifier = (1.0 / left_dist_sq).min(1.0);
        let right_dist_modifier = (1.0 / right_dist_sq).min(1.0);
        self.3[0] = left_diff_modifier * left_dist_modifier;
        self.3[1] = right_diff_modifier * right_dist_modifier;

    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        self.0
            .sample(input)
            .map(|x| x * self.3[input.channel as usize])
    }

    fn duration(&self) -> Option<std::time::Duration> {
        self.0.duration()
    }

    fn target_channels(&self) -> Option<u16> {
        self.0.target_channels().map(|_| 2)
    }

    fn target_sample_rate(&self) -> Option<u32> {
        self.0.target_sample_rate()
    }
}
