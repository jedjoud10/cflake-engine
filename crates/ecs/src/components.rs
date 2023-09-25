use std::borrow::Cow;

use crate::Component;

/// Default name component
#[derive(Component)]
pub struct Named(pub Cow<'static, str>);

/// Default tag component
#[derive(Component)]
pub struct Tagged(pub Cow<'static, str>);