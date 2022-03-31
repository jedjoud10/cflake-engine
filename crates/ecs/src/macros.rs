#[macro_export]
macro_rules! layout {
    ($($x:ty),*) => {
        // Simple array of masks
        {
            let mut mask = $crate::Mask::default();
            $(
                // Add the mask
                mask = mask | $crate::registry::mask::<$x>().unwrap();
            )*
            mask
        }
    }
}
