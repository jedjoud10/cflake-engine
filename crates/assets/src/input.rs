use crate::Asset;

// Asset path input that might also contain the asset custom loading arguments
pub trait AssetInput<'str, 'ctx, 'stg, A: Asset> {
    fn split(self) -> (&'str str, A::Settings<'stg>, A::Context<'ctx>);
    fn path(&self) -> &'str str;
}

// No context nor settings, assumes both are Default
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A> for &'str str
where
    A::Context<'ctx>: Default,
    A::Settings<'stg>: Default,
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Settings<'stg>,
        <A as Asset>::Context<'ctx>,
    ) {
        (self, Default::default(), Default::default())
    }

    fn path(&self) -> &'str str {
        self
    }
}

// Contains the context only, assumes the settings is Default-able
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A> for (&'str str, A::Context<'ctx>)
where
    <A as Asset>::Settings<'stg>: Default,
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Settings<'stg>,
        <A as Asset>::Context<'ctx>,
    ) {
        (self.0, Default::default(), self.1)
    }

    fn path(&self) -> &'str str {
        self.0
    }
}

// Contains both the context and settings
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A>
    for (&'str str, A::Settings<'stg>, A::Context<'ctx>)
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Settings<'stg>,
        <A as Asset>::Context<'ctx>,
    ) {
        self
    }

    fn path(&self) -> &'str str {
        self.0
    }
}
