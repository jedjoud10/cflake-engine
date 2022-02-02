// Some main settings for the mesher
#[derive(Clone, Copy)]
pub struct MesherSettings {
    pub interpolation: bool,
    pub skirts: bool,
}