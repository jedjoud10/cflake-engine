// Some main settings for the mesher
#[derive(Clone, Copy)]
pub struct MesherSettings {
    pub(crate) interpolation: bool,
    pub(crate) skirts: bool,
}