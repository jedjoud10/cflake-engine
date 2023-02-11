use crate::Texel;

// What we should do when loading in data from the attachment
// Even though WGPU has a LoadOp type, I still decided to implement one myself simply
// due to the fact that we can use type safety to store the texel color type
pub enum LoadOp<T: Texel> {
    Load,
    Clear(T::Storage),
}

// What we should do when writing data to the attachment
pub enum StoreOp {
    Ignore,
    Store,
}
