use crate::{AnyElement, Normalized, Texel, BGRA, R, RG, RGBA, Conversion};

// Color texels are texels used for color attachments
// TODO: Figure out if there are any limits to this
// TODO: Maybe add saturation / hue control (since this is strictly color after all)
pub trait ColorTexel: Texel + Conversion<Target = vek::Vec4<f32>> {
    // Convert this texel to the wgpu color struct (if possible)
    // This returns None if the color data is not in the 0 - 1 range or simply not color data
    fn try_into_color(texel: Self::Storage) -> Option<wgpu::Color> {
        let rgba = Self::into_target(texel);

        // If any value is greater than 1 or less than 0, then it cannot be
        // represented as a wgpu color (it actually can but wtv who cares)
        if rgba.reduce_partial_max() > 1.0
            || rgba.reduce_partial_min() < 0.0
        {
            return None;
        }

        Some(wgpu::Color {
            r: rgba.x as f64,
            g: rgba.y as f64,
            b: rgba.z as f64,
            a: rgba.w as f64,
        })
    }

    // Linearly interpolate between two color texels
    fn lerp(a: Self::Storage, b: Self::Storage, t: f32) -> Self::Storage {
        let a = Self::into_target(a);
        let b = Self::into_target(b);
        let c = a * (1.0 - t) + b * t;
        Self::try_from_target(c).unwrap()
    }
}

impl<T: Texel + Conversion<Target = vek::Vec4<f32>>> ColorTexel for T {}