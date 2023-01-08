use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DescriptorSet, attributes(sampler))]
pub fn derive_descriptor_set(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs,
        vis,
        ident,
        generics,
        data
    } = parse_macro_input!(input as DeriveInput);

    //let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let output = quote! {
        impl<'a> DescriptorLayout for #ident<'a> {
        }
        
        impl<'a> DescriptorSet<'a> for #ident<'a> {
        }
        
    };
    output.into()
}