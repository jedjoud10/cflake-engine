use std::{num::NonZeroU8, ops::Add};

use crate::Graphics;

pub use wgpu::TextureViewDimension;
pub use wgpu::TextureDimension;

// Texture dimensions traits that are simply implemented for extents
pub trait Extent: Copy + Send + Sync {
    // Get the max element from these dimensions
    fn reduce_max(&self) -> u32;

    // Get the min element from these dimensions
    fn reduce_min(&self) -> u32;

    // Check if this region can be used to create a texture or update it
    fn is_valid(&self) -> bool {
        self.reduce_min() > 0
    }

    // Calculate the dimensions of a mip map level using it's index
    // Level equal to 0 meaning that it will return the base extent
    fn mip_level_dimensions(self, level: u8) -> Self;

    // Check if an extent is larger in all axii than another one
    fn is_larger_than(self, other: Self) -> bool;

    // Get the width of the extent
    fn width(&self) -> u32;

    // Get the height of the extent
    fn height(&self) -> u32;

    // Depth or layers of this extent
    fn depth_or_layers(&self) -> u32;

    // Create a new extent by cloning a value for all axii
    fn broadcast(val: u32) -> Self;

    // Create a new extent using a width, height, depth
    fn new(w: u32, h: u32, d: u32) -> Self;

    // Convert the extent to Extent3D
    fn decompose(&self) -> vek::Extent3<u32> {
        vek::Extent3::new(self.width(), self.height(), self.depth_or_layers())
    }
}

// Texture offsets traits that are simply implemented for origins
pub trait Origin: Copy + Default {
    // Get the X offset of the origin
    fn x(&self) -> u32;

    // Get the Y offset of the origin
    fn y(&self) -> u32;

    // Get the Z offset of the origin (3d texture)
    fn z(&self) -> u32;

    // Get the layer offset of the origin (layered textures)
    fn layer(&self) -> u32;
}

// Only used for layered textures for their views
pub trait LayeredOrigin: Copy + Default + Origin {}

// Implementation of extent for 2D extent
impl Extent for vek::Extent2<u32> {
    fn reduce_max(&self) -> u32 {
        vek::Extent2::reduce_max(*self)
    }

    fn reduce_min(&self) -> u32 {
        vek::Extent2::reduce_min(*self)
    }

    fn is_larger_than(self, other: Self) -> bool {
        self.cmpge(&other).reduce_and()
    }

    fn width(&self) -> u32 {
        self.w
    }

    fn height(&self) -> u32 {
        self.h
    }

    fn depth_or_layers(&self) -> u32 {
        1
    }

    fn broadcast(val: u32) -> Self {
        vek::Extent2::broadcast(val)
    }

    fn new(w: u32, h: u32, _: u32) -> Self {
        vek::Extent2::new(w, h)
    }

    fn mip_level_dimensions(self, level: u8) -> Self {
        self / 2u32.pow(level as u32)
    }
}

// Implementation of extent for 3D extent
impl Extent for vek::Extent3<u32> {
    fn reduce_max(&self) -> u32 {
        vek::Extent3::reduce_max(*self)
    }

    fn reduce_min(&self) -> u32 {
        vek::Extent3::reduce_min(*self)
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent3::zero()
    }

    fn is_larger_than(self, other: Self) -> bool {
        self.cmpge(&other).reduce_and()
    }

    fn width(&self) -> u32 {
        self.w
    }

    fn height(&self) -> u32 {
        self.h
    }

    fn depth_or_layers(&self) -> u32 {
        self.d
    }

    fn broadcast(val: u32) -> Self {
        vek::Extent3::broadcast(val)
    }

    fn new(w: u32, h: u32, d: u32) -> Self {
        vek::Extent3::new(w, h, d)
    }

    fn mip_level_dimensions(self, level: u8) -> Self {
        self / 2u32.pow(level as u32)
    }
}

// Implementation of extent for 2D layered texture extent
impl Extent for (vek::Extent2<u32>, u32) {
    fn reduce_max(&self) -> u32 {
        vek::Extent2::new(self.0.w, self.0.h).reduce_max()
    }

    fn reduce_min(&self) -> u32 {
        vek::Extent2::new(self.0.w, self.0.h).reduce_min()
    }

    fn is_valid(&self) -> bool {
        self.0 != vek::Extent2::zero() && self.1 != 0
    }

    fn is_larger_than(self, other: Self) -> bool {
        self.0.cmpge(&other.0).reduce_and() && self.1 > other.1
    }

    fn width(&self) -> u32 {
        self.0.w
    }

    fn height(&self) -> u32 {
        self.0.h
    }

    fn depth_or_layers(&self) -> u32 {
        self.1
    }

    fn broadcast(val: u32) -> Self {
        (vek::Extent2::broadcast(val), val)
    }

    fn new(w: u32, h: u32, d: u32) -> Self {
        (vek::Extent2::new(w, h), d)
    }

    fn mip_level_dimensions(self, level: u8) -> Self {
        (self.0 / 2u32.pow(level as u32), self.1)
    }
}

// Implementation of origin for 2D vec
impl Origin for vek::Vec2<u32> {
    fn x(&self) -> u32 {
        self.x
    }

    fn y(&self) -> u32 {
        self.y
    }

    fn z(&self) -> u32 {
        0
    }

    fn layer(&self) -> u32 {
        0
    }
}

// Implementation of origin for 3D vec
impl Origin for vek::Vec3<u32> {
    fn x(&self) -> u32 {
        self.x
    }

    fn y(&self) -> u32 {
        self.y
    }

    fn z(&self) -> u32 {
        self.z
    }

    fn layer(&self) -> u32 {
        0
    }
}

// Implementation of origin for layered 2D / cubemap
impl Origin for (vek::Vec2<u32>, u32) {
    fn x(&self) -> u32 {
        self.0.x
    }

    fn y(&self) -> u32 {
        self.0.y
    }

    fn z(&self) -> u32 {
        0
    }

    fn layer(&self) -> u32 {
        self.1
    }
}
impl LayeredOrigin for (vek::Vec2<u32>, u32) {}

// Texture region trait that will be implemented for (origin, extent) tuples
pub trait Region: Copy {
    // Regions are defined by their origin and extents
    type O: Origin + Default + Copy + std::fmt::Debug;
    type E: Extent + Copy + PartialEq + std::fmt::Debug;

    // Create a texel region of one singular unit (so we can store a singular texel)
    fn unit() -> Self;

    // Get the region's origin
    fn origin(&self) -> Self::O;

    // Get the region's extent
    fn extent(&self) -> Self::E;

    // Set the region's origin
    fn set_origin(&mut self, origin: Self::O);

    // Set the region's extent
    fn set_extent(&mut self, extent: Self::E);

    // Create a region with it's raw components
    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self;

    // Create a region with a zeroed out origin and specific extent
    fn from_extent(extent: Self::E) -> Self {
        Self::from_raw_parts(<Self::O as Default>::default(), extent)
    }

    // Is this region a multi-layer region (cubemap / layered texture)
    fn is_multi_layered() -> bool {
        match Self::view_dimension() {
            wgpu::TextureViewDimension::D2Array | wgpu::TextureViewDimension::CubeArray => true,
            _ => false,
        }
    }

    // Get the number of layers of an extent
    fn layers(extent: Self::E) -> u32;

    // TODO: kill me
    fn depth_or_layers(extent: Self::E) -> u32 {
        match Self::view_dimension() {
            wgpu::TextureViewDimension::D1 => 1,
            wgpu::TextureViewDimension::D2 => 1,
            wgpu::TextureViewDimension::D2Array => extent.depth_or_layers(),
            wgpu::TextureViewDimension::Cube => 6,
            wgpu::TextureViewDimension::CubeArray => 6 * extent.depth_or_layers(),
            wgpu::TextureViewDimension::D3 => extent.depth_or_layers(),
        }
    }

    // Check if this region is larger than another region
    // Aka if the "other" region fits within self
    fn is_larger_than(self, other: Self) -> bool;

    // Check if we can use a texture mip level (with specific region of Self) as a render target directly
    fn can_render_to_mip(&self) -> bool {
        // works for now ig
        Self::depth_or_layers(self.extent()) == 1
        /*
        let depth_or_layers = self.extent().depth_or_layers();

        match Self::view_dimension() {
            wgpu::TextureViewDimension::D2Array => todo!(),
            wgpu::TextureViewDimension::Cube => todo!(),
            wgpu::TextureViewDimension::CubeArray => todo!(),
            wgpu::TextureViewDimension::D3 => todo!(),
        }

        let layer_check = self.extent().layers().checked_sub(self.origin().layer());
        let depth_check = self.extent().depth().checked_sub(self.origin().z());

        let (Some(layers), Some(depth)) = (layer_check, depth_check) else {
            return false
        };

        return layers == 1 && depth == 1;
        */
    }

    // Check if the extent is a power of 2 extent
    fn is_power_of_two(extent: Self::E) -> bool {
        match Self::dimension() {
            TextureDimension::D1 => extent.width().is_power_of_two(),
            TextureDimension::D2 => extent.width().is_power_of_two() && extent.height().is_power_of_two(),
            TextureDimension::D3 => {
                extent.width().is_power_of_two()
                    && extent.height().is_power_of_two()
                    && extent.depth_or_layers().is_power_of_two()
            }
        }
    }

    // Caclulate the number of mipmap levels that a texture can have
    // Returns None if the extent is a NPOT extent
    // Returns 1 if the texture only has one mip
    fn levels(extent: Self::E) -> Option<NonZeroU8> {
        if !Self::is_power_of_two(extent) {
            return None;
        }

        let cur = extent.reduce_min() as f32;
        let num = cur.log2().floor();
        Some(
            NonZeroU8::new(u8::try_from(num as u8 + 1).unwrap())
                .unwrap_or(NonZeroU8::new(1).unwrap()),
        )
    }

    // Calculate the number of texels inside a region (extent)
    fn volume(extent: Self::E) -> u32;

    // Get the view dimensions of the extent (1, 2, 3, or layered / cube maps)
    fn view_dimension() -> TextureViewDimension;

    // Get the dimensionality of the underlying texels (1, 2, 3)
    fn dimension() -> TextureDimension {
        match Self::view_dimension() {
            TextureViewDimension::D1 => TextureDimension::D1,
            TextureViewDimension::D2 => TextureDimension::D2,
            TextureViewDimension::D2Array => TextureDimension::D2,
            TextureViewDimension::Cube => TextureDimension::D2,
            TextureViewDimension::CubeArray => TextureDimension::D2,
            TextureViewDimension::D3 => TextureDimension::D3,
        }
    }
}

// Texture2D
impl Region for (vek::Vec2<u32>, vek::Extent2<u32>) {
    type O = vek::Vec2<u32>;
    type E = vek::Extent2<u32>;

    fn unit() -> Self {
        (vek::Vec2::zero(), vek::Extent2::one())
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_larger_than(self, other: Self) -> bool {
        let e = other.extent() + vek::Extent2::<u32>::from(other.origin());
        self.extent().is_larger_than(e)
    }

    fn volume(extent: Self::E) -> u32 {
        extent.product()
    }

    fn view_dimension() -> TextureViewDimension {
        TextureViewDimension::D2
    }

    fn layers(extent: Self::E) -> u32 {
        1
    }
}

// Texture3D
impl Region for (vek::Vec3<u32>, vek::Extent3<u32>) {
    type O = vek::Vec3<u32>;
    type E = vek::Extent3<u32>;

    fn unit() -> Self {
        (vek::Vec3::zero(), vek::Extent3::one())
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_larger_than(self, other: Self) -> bool {
        let e = other.extent() + vek::Extent3::<u32>::from(other.origin());
        self.extent().is_larger_than(e)
    }

    fn volume(extent: Self::E) -> u32 {
        extent.product()
    }

    fn view_dimension() -> TextureViewDimension {
        TextureViewDimension::D3
    }

    fn layers(extent: Self::E) -> u32 {
        1
    }
}

// CubeMap2D
impl Region for ((vek::Vec2<u32>, u32), vek::Extent2<u32>) {
    type O = (vek::Vec2<u32>, u32);
    type E = vek::Extent2<u32>;

    fn unit() -> Self {
        ((vek::Vec2::zero(), 0), vek::Extent2::one())
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_larger_than(self, other: Self) -> bool {
        if self.origin().1 >= 6 {
            return false;
        }

        let e = other.extent() + vek::Extent2::<u32>::from(other.origin().0);
        self.extent().is_larger_than(e)
    }

    fn volume(extent: Self::E) -> u32 {
        // The extent is 2D, so this is basically just the area
        extent.product() * 6
    }

    fn view_dimension() -> TextureViewDimension {
        TextureViewDimension::Cube
    }

    fn layers(extent: Self::E) -> u32 {
        6
    }
}

// LayeredTexture2D
impl Region for ((vek::Vec2<u32>, u32), (vek::Extent2<u32>, u32)) {
    type O = (vek::Vec2<u32>, u32);
    type E = (vek::Extent2<u32>, u32);

    fn unit() -> Self {
        ((vek::Vec2::zero(), 0), (vek::Extent2::one(), 1))
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_larger_than(self, other: Self) -> bool {
        if self.origin().1 >= 6 {
            return false;
        }

        let e = other.extent().0 + vek::Extent2::<u32>::from(other.origin().0);
        self.1 .0.is_larger_than(e) && self.1 .1 > other.1 .1
    }

    fn volume(extent: Self::E) -> u32 {
        extent.0.product() * extent.1
    }

    fn view_dimension() -> TextureViewDimension {
        TextureViewDimension::D2Array
    }

    fn layers(extent: Self::E) -> u32 {
        extent.depth_or_layers()
    }
}
