macro_rules! tuple_impls_layout {
    ( $( $name:ident )+, $max:tt) => {
    };
}

tuple_impls_layout! { C0 C1, 2 }
tuple_impls_layout! { C0 C1 C2, 3 }
tuple_impls_layout! { C0 C1 C2 C3, 4 }
tuple_impls_layout! { C0 C1 C2 C3 C4, 5 }