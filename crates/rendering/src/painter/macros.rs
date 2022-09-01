use super::{ToCanvasStorage, CanvasLayout, CanvasStorage};
use seq_macro::seq;

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt) => {
        impl<'a, $($name: ToCanvasStorage<'a>),+> CanvasLayout<'a> for ($($name,)+) {
            fn storages(&self) -> Vec<CanvasStorage> {
                // TODO: Remove this hack
                let mut vec = Vec::<CanvasStorage>::new();
                seq!(N in 0..$max {
                    let desc = ToCanvasStorage::into(&self.N);
                    vec.push(desc);
                });
                vec
            }
        }
    };
}

tuple_impls_color_layout! { C0 C1, 2  }
tuple_impls_color_layout! { C0 C1 C2, 3 }
tuple_impls_color_layout! { C0 C1 C2 C3, 4 }
tuple_impls_color_layout! { C0 C1 C2 C3 C4, 5 }
