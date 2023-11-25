use std::marker::PhantomData;

use crate::format::{AnyElement, Base, Normalizable, Normalized, SupportsSrgba};
use paste::paste;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompressionType {
    // RGBA<Normalized<UBC1>> R5G6B5A1
    // SRGBA<Normalized<UBC1>> R5G6B5A1
    UBC1,

    // RGBA<Normalized<UBC2>> R5G6B5A4
    // SRGBA<Normalized<UBC2>> R5G6B5A4
    UBC2,

    // RGBA<Normalized<UBC3>> R5G6B5A8
    // SRGBA<Normalized<UBC3>> R5G6B5A8
    UBC3,

    // R<Normalized<UBC4>>
    // R<Normalized<SBC4>>
    BC4 { signed: bool },

    // RG<Normalized<UBC5>>
    // RG<Normalized<SBC5>>
    BC5 { signed: bool },

    // RGBA<Normalized<UBC7>>
    // SRGBA<Normalized<UBC7>>
    UBC7,
}

impl CompressionType {
    // Get the compression block size for the current compression type
    pub fn block_size(self) -> u32 {
        4
    }

    // Get the number of bytes per block
    pub fn bytes_per_block(self) -> u32 {
        match self {
            CompressionType::UBC1 => 8,
            CompressionType::UBC2 => 16,
            CompressionType::UBC3 => 16,
            CompressionType::BC4 { .. } => 8,
            CompressionType::BC5 { .. } => 16,
            CompressionType::UBC7 => 16,
        }
    }

    // Get the number of bits per pixel
    pub fn bits_per_pixel(self) -> u32 {
        self.bytes_per_block() / 2
    }
}

macro_rules! impl_compressed_any_element {
    ($t:ty, $storage:ty, $variant:expr) => {
        impl AnyElement for $t {
            type Storage = $storage;
            const ELEMENT_TYPE: crate::format::ElementType = crate::format::ElementType::Compressed($variant);
        }

        impl Normalizable for $t {}
    };
}

// In WGPU, only specific data types support compression
pub struct UBC1(PhantomData<u8>);
pub struct UBC2(PhantomData<u8>);
pub struct UBC3(PhantomData<u8>);
pub struct UBC4(PhantomData<u8>);
pub struct SBC4(PhantomData<u8>);
pub struct UBC5(PhantomData<u8>);
pub struct SBC5(PhantomData<u8>);
pub struct UBC7(PhantomData<u8>);

impl_compressed_any_element!(UBC1, u8, CompressionType::UBC1);
impl_compressed_any_element!(UBC2, u8, CompressionType::UBC2);
impl_compressed_any_element!(UBC3, u8, CompressionType::UBC3);
impl_compressed_any_element!(UBC4, u8, CompressionType::BC4 { signed: false });
impl_compressed_any_element!(SBC4, i8, CompressionType::BC4 { signed: true });
impl_compressed_any_element!(UBC5, u8, CompressionType::BC5 { signed: false });
impl_compressed_any_element!(SBC5, i8, CompressionType::BC5 { signed: true });
impl_compressed_any_element!(UBC7, u8, CompressionType::UBC7);

impl SupportsSrgba for Normalized<UBC1> {}
impl SupportsSrgba for Normalized<UBC2> {}
impl SupportsSrgba for Normalized<UBC3> {}
impl SupportsSrgba for Normalized<UBC7> {}
