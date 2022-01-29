use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, .. } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        // Main traits implemented
        impl #impl_generics Component for #ident #ty_generics #where_clause {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn get_component_name() -> String {
                String::from(stringify!(#ident).split(" ").last().unwrap().to_string())
            }
        }
    };
    output.into()
}