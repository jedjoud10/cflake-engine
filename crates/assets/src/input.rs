use crate::Asset;

// Asset path input that might also contain the asset custom loading arguments
pub trait AssetInput<'s, 'args, A: Asset> {
    fn split(self) -> (&'s str, A::Args<'args>);
    fn path(&self) -> &'s str;
}

impl<'s, 'args, A: Asset> AssetInput<'s, 'args, A> for &'s str
where
    A::Args<'args>: Default,
{
    fn split(self) -> (&'s str, <A as Asset>::Args<'args>) {
        (self, A::Args::default())
    }

    fn path(&self) -> &'s str {
        self
    }
}

impl<'s, 'args, A: Asset> AssetInput<'s, 'args, A> for (&'s str, A::Args<'args>) {
    fn split(self) -> (&'s str, <A as Asset>::Args<'args>) {
        self
    }

    fn path(&self) -> &'s str {
        self.0
    }
}
