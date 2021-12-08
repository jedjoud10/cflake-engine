#[macro_export]
macro_rules! impl_custom_system_data {
    ($t: ty) => {
        impl CustomSystemData for $t {
        }
    };
}
