#[cfg(test)]
mod tests {
    use assets::Asset;

    use crate::{Mesh, Buffer};

    #[test]
    fn test() {
        //let test: assets::loader::AssetLoader = todo!();

        pub struct AttributeSet {
            positions: Option<Buffer<vek::Vec3<f32>>>,
            normals: Option<Buffer<vek::Vec3<i8>>>,
            tangents: Option<Buffer<vek::Vec4<i8>>>,
            colors: Option<Buffer<vek::Rgb<u8>>>,
            tex_coord: Option<Buffer<vek::Vec2<u8>>>,
        }

        //let mesh = Mesh::try_load
        dbg!(std::mem::size_of::<AttributeSet>());
    }
}
