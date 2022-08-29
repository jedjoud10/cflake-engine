use super::{DisplayStorageDescriptor, ScopedCanvasLayout, ToDisplayStorageDescriptor};
use seq_macro::seq;

// Default scoped canvas layout does not contain any attachments
impl<'a> ScopedCanvasLayout<'a> for () {
    fn descriptors(&self) -> Vec<DisplayStorageDescriptor<'a>> {
        panic!()
    }

    fn is_valid(&self) -> bool {
        true
    }
}

macro_rules! tuple_impls_color_layout {
    ( $( $name:ident )+, $max:tt) => {
        impl<'a, $($name: ToDisplayStorageDescriptor<'a>),+> ScopedCanvasLayout<'a> for ($($name,)+) {
            fn descriptors(&self) -> Vec<DisplayStorageDescriptor<'a>> {
                // TODO: Remove this hack
                let mut vec = Vec::<DisplayStorageDescriptor<'a>>::new();
                seq!(N in 0..$max {
                    let desc = ToDisplayStorageDescriptor::into(&self.N);
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
