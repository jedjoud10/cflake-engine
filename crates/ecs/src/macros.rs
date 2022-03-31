#[macro_export]
macro_rules! layout {
    ($($x:ty),*) => {
        // Simple array of masks
        {
            let mut mask = $crate::prelude::Mask::default();
            $(
                // Add the mask
                mask = mask | $crate::prelude::registry::mask::<$x>().unwrap();
            )*
            mask
        }
    }
}
