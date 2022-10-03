use std::num::NonZeroU8;

// Texture dimensions traits that are simply implemented for extents
pub trait Extent: Copy {
    // Get the surface area of a superficial rectangle that uses these extents as it's dimensions
    fn area(&self) -> u32;

    // Check if this region can be used to create a texture or update it
    fn is_valid(&self) -> bool;

    // Get the max element from these dimensions
    fn max(&self) -> u16;

    // Caclulate the number of mipmap levels that a texture can have
    fn levels(&self) -> NonZeroU8 {
        let mut cur = self.max() as f32;
        let mut num = 0u32;
        while cur > 1.0 {
            cur /= 2.0;
            num += 1;
        }
        NonZeroU8::new(u8::try_from(num).unwrap()).unwrap_or(NonZeroU8::new(1).unwrap())
    }

    // Check if this extent is smaller than another extent (in all axii)
    fn is_self_smaller(&self, other: Self) -> bool;

    // Get the dimensions of a specific miplayer using raw OpenGL commands
    unsafe fn get_level_extent(texture: u32, level: u8) -> Self;
}

// Implementation of extent for 2D extent
impl Extent for vek::Extent2<u16> {
    fn area(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent2::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }

    fn is_self_smaller(&self, other: Self) -> bool {
        self.cmple(&other).reduce_and()
    }

    unsafe fn get_level_extent(texture: u32, level: u8) -> Self {
        let mut width = 0;
        let mut height = 0;
        gl::GetTextureLevelParameteriv(texture, level as i32, gl::TEXTURE_WIDTH, &mut width);
        gl::GetTextureLevelParameteriv(texture, level as i32, gl::TEXTURE_HEIGHT, &mut height);
        Self::new(width as u16, height as u16)
    }
}

// Implementation of extent for 3D extent
impl Extent for vek::Extent3<u16> {
    fn area(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent3::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }

    fn is_self_smaller(&self, other: Self) -> bool {
        self.cmple(&other).reduce_and()
    }

    unsafe fn get_level_extent(texture: u32, level: u8) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut depth = 0;
        gl::GetTextureLevelParameteriv(texture, level as i32, gl::TEXTURE_WIDTH, &mut width);
        gl::GetTextureLevelParameteriv(texture, level as i32, gl::TEXTURE_HEIGHT, &mut height);
        gl::GetTextureLevelParameteriv(texture, level as i32, gl::TEXTURE_DEPTH, &mut depth);
        Self::new(width as u16, height as u16, depth as u16)
    }
}
