use crate::Texel;

// What we should do when loading in data from the attachment
pub enum LoadOp<T: Texel> {
    Ignore,
    Load,
    Clear(T::Storage),
}

// What we should do when writing data to the attachment
pub enum StoreOp {
    Ignore,
    Store,
}
