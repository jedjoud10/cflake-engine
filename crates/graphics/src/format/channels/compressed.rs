use crate::{Normalized, Normalizable, Base};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompressionType {
    // RGBA<Normalized<UBC2>> R5G6B5A1
    // SRGBA<Normalized<UBC2>> R5G6B5A1
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
    UBC7
}

// In WGPU, only specific data types support compression
pub struct UBC1;

impl Base for UBC1 {
    const TYPE: crate::BaseType = crate::BaseType::;

    const SIGNED: bool = false;
}

pub struct UBC2;
pub struct UBC3;
pub struct UBC4;
pub struct SBC4;
pub struct UBC5;
pub struct SBC5;
pub struct UBC7;




