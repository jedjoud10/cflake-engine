use crate::Component;

/// Default name component
#[derive(Component)]
pub struct Named(pub String);

/// Default tag component
#[derive(Component)]
pub struct Tagged(pub String);
