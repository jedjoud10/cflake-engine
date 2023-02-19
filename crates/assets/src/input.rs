use crate::Asset;

// Asset path input that might also contain the asset custom loading arguments
pub trait AssetInput<'str, 'ctx, 'stg, A: Asset> {
    fn split(
        self,
    ) -> (&'str str, A::Context<'ctx>, A::Settings<'stg>);
    fn path(&self) -> &'str str;
}

// No context nor settings, assumes both are Default
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A>
    for &'str str
where
    A::Context<'ctx>: Default,
    A::Settings<'stg>: Default,
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Context<'ctx>,
        <A as Asset>::Settings<'stg>,
    ) {
        (self, Default::default(), Default::default())
    }

    fn path(&self) -> &'str str {
        self
    }
}

// Contains the context only, assumes the settings is Default-able
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A>
    for (&'str str, A::Context<'ctx>)
where
    <A as Asset>::Settings<'stg>: Default,
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Context<'ctx>,
        <A as Asset>::Settings<'stg>,
    ) {
        (self.0, self.1, Default::default())
    }

    fn path(&self) -> &'str str {
        self.0
    }
}

// Contains both the context and settings
impl<'str, 'ctx, 'stg, A: Asset> AssetInput<'str, 'ctx, 'stg, A>
    for (&'str str, A::Context<'ctx>, A::Settings<'stg>)
{
    fn split(
        self,
    ) -> (
        &'str str,
        <A as Asset>::Context<'ctx>,
        <A as Asset>::Settings<'stg>,
    ) {
        self
    }

    fn path(&self) -> &'str str {
        self.0
    }
}
