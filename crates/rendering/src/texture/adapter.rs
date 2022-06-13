use super::{Base, R, RG, RGB, RGBA, Limiter, Ranged, Normalized};


pub trait TexelAdapter {
    // The data that is actually stored within the texels
    type Raw;

    // The data that the user will interact with (input, output)
    type User;

    // Convert the raw texel data to it's user repr
    fn raw_to_user(raw: Self::Raw) -> Self::User;

    // Convert the user texel data to it's raw repr
    fn raw_from_user(user: Self::User) -> Self::Raw;

    // Check if a user texel can be converted to it's raw counter part or not
    fn can_convert_to_raw(user: Self::User) -> bool;
}

macro_rules! impl_adapter_base {
    ($elem: ident) => {
        paste::paste! {
            impl TexelAdapter for R<[<$elem>]> {
                type Raw = $elem;            
                type User = $elem;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    true
                }
            }
            
            impl TexelAdapter for RG<[<$elem>]> {
                type Raw = vek::Vec2<$elem>;            
                type User = vek::Vec2<$elem>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    true
                }
            }
            
            impl TexelAdapter for RGB<[<$elem>]> {
                type Raw = vek::Vec3<$elem>;            
                type User = vek::Vec3<$elem>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    true
                }
            }
            
            impl TexelAdapter for RGBA<[<$elem>]> {
                type Raw = vek::Vec4<$elem>;            
                type User = vek::Vec4<$elem>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    true
                }
            }
        }
    };
}

macro_rules! impl_adapter_limiter {
    ($elem: ident, $limiter: ident) => {
        paste::paste! {
            impl TexelAdapter for R<$limiter<$elem>> {
                type Raw = $elem;            
                type User = f32;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    $limiter::<[<$elem>]>::inner_into_f32(raw)
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    $limiter::<[<$elem>]>::inner_from_f32(user)
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    $limiter::<[<$elem>]>::in_range(user)
                }
            }

            impl TexelAdapter for RG<$limiter<$elem>> {
                type Raw = vek::Vec2<$elem>;            
                type User = vek::Vec2<f32>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw.map($limiter::<[<$elem>]>::inner_into_f32)
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user.map($limiter::<[<$elem>]>::inner_from_f32)
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    let r = $limiter::<[<$elem>]>::in_range(user.x);
                    let g = $limiter::<[<$elem>]>::in_range(user.y);
                    r && g
                }
            }

            impl TexelAdapter for RGB<$limiter<$elem>> {
                type Raw = vek::Vec3<$elem>;            
                type User = vek::Vec3<f32>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw.map($limiter::<[<$elem>]>::inner_into_f32)
                }
            
                fn raw_from_user(user: Self::User) -> Self::Raw {
                    user.map($limiter::<[<$elem>]>::inner_from_f32)
                }

                fn can_convert_to_raw(user: Self::User) -> bool {
                    let r = $limiter::<[<$elem>]>::in_range(user.x);
                    let g = $limiter::<[<$elem>]>::in_range(user.y);
                    let b = $limiter::<[<$elem>]>::in_range(user.z);
                    r && g && b
                }
            }
            
            /*
            impl TexelAdapter for RG<[<$elem>]> {
                type Raw = vek::Vec2<$elem>;            
                type User = vek::Vec2<f32>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                unsafe fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }
            }
            
            impl TexelAdapter for RGB<[<$elem>]> {
                type Raw = vek::Vec3<$elem>;            
                type User = vek::Vec3<f32>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                unsafe fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }
            }
            
            impl TexelAdapter for RGBA<[<$elem>]> {
                type Raw = vek::Vec4<$elem>;            
                type User = vek::Vec4<f32>;
            
                fn raw_to_user(raw: Self::Raw) -> Self::User {
                    raw
                }
            
                unsafe fn raw_from_user(user: Self::User) -> Self::Raw {
                    user
                }
            }
            */
        }
    };
}

impl_adapter_base!(u8);
impl_adapter_base!(i8);
impl_adapter_base!(u16);
impl_adapter_base!(i16);
impl_adapter_base!(u32);
impl_adapter_base!(i32);

impl_adapter_limiter!(u8, Ranged);