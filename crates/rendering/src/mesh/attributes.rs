use std::{ptr::null, mem::{MaybeUninit, size_of}, any::TypeId};

use crate::{
    buffer::{ArrayBuffer, BufferMode},
    object::{Shared, ToGlName},
};
use super::{Mesh, EnabledAttributes};


/*
impl Attribute for Position {
    type Out = vek::Vec3<f32>;
    const ENABLED: MeshBuffers = MeshBuffers::POSITIONS;
    const NORMALIZED: bool = false;
    
    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.positions.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.positions = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        panic!()
    }    
}

impl Attribute for Normal {
    type Out = vek::Vec3<i8>;
    const ENABLED: MeshBuffers = MeshBuffers::NORMALS;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.normals.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.normals = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nbv(Self::attribute_index(), [127, 127, 127, 0_i8].as_ptr());
    }
}

impl Attribute for Tangent {
    type Out = vek::Vec4<i8>;
    const ENABLED: MeshBuffers = MeshBuffers::TANGENTS;
    const NORMALIZED: bool= true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tangents.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tangents = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nbv(Self::attribute_index(), [0, 0, 0, 127_i8].as_ptr());
    }
}

impl Attribute for Color {
    type Out = vek::Rgb<u8>;
    const ENABLED: MeshBuffers = MeshBuffers::COLORS;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.colors.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.colors = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nub(Self::attribute_index(), 255, 255, 255, 0);
    }
}

impl Attribute for TexCoord {
    type Out = vek::Vec2<u8>;
    const ENABLED: MeshBuffers = MeshBuffers::TEX_COORD;
    const NORMALIZED: bool = true;

    unsafe fn assume_init_get(mesh: &Mesh) -> &ArrayBuffer<Self::Out> {
        mesh.tex_coord.assume_init_ref()
    }

    unsafe fn assume_init_get_mut(mesh: &mut Mesh) -> &mut ArrayBuffer<Self::Out> {
        mesh.tex_coord.assume_init_mut()
    }

    unsafe fn set_raw(mesh: &mut Mesh, buffer: ArrayBuffer<Self::Out>) {
        mesh.tex_coord = MaybeUninit::new(buffer);
    }

    unsafe fn default() {
        gl::VertexAttrib4Nub(Self::attribute_index(), 255, 255, 0, 0);
    }
}

// All the raw types used by the attributes
pub type VePosition = <Position as Attribute>::Out;
pub type VeNormal = <Normal as Attribute>::Out;
pub type VeTangent = <Tangent as Attribute>::Out;
pub type VeColor = <Color as Attribute>::Out;
pub type VeTexCoord0 = <TexCoord as Attribute>::Out;
*/