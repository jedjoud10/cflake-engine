use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Resource, attributes(Locked))]
pub fn derive_resources(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        generics,
        attrs,
        ..
    } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let removable = attrs.is_empty();
    let output = quote! {
        impl #impl_generics Resource for #ident #ty_generics #where_clause {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn can_remove() -> bool {
                #removable
            }
        }
    };
    output.into()
}
