use crate::{advanced::tracker::MaybeGlTracker, object::OpenGLObjectNotInitialized};

// Writable
pub trait Writable
where
    Self: Sized,
{
    type Data;
    fn glwrite(&mut self, input: Self::Data) -> MaybeGlTracker<Self, ()>;
}
// Readable
pub trait Readable
where
    Self: Sized,
{
    type Data;
    fn glread(&mut self) -> MaybeGlTracker<Self, Self::Data>;
}
