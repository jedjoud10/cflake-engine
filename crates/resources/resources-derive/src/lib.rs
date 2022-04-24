use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Resource)]
pub fn derive_global(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, .. } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        // Main traits implemented
        impl #impl_generics Resource for #ident #ty_generics #where_clause {
        }
    };
    output.into()
}