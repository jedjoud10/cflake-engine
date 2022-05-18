use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Uniform)]
pub fn derive_components(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, data, attrs, .. } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let syn::Data::Struct(data) = data {
        if let syn::Fields::Named(fields) = data.fields {
            let data_expanded_members = fields.named.into_iter().map(|field| {
                let field_name = field.ident.expect("Unreachable");
                let span = field_name.span();
                let field_name_stringified =
                    LitStr::new(&field_name.to_string(), span)
                ;
                quote_spanned! { span=>
                    make_number(#field_name_stringified, &self.#field_name)
                }
            }); // : impl Iterator<Item = TokenStream2>
        }
    }
    let output = quote! {
        impl MyTrait for #ident { // I can access name, yeah!
        }
        // Uniforms are created using the proper Uniform derive macro
        // The macro will take any struct, and use it as uniforms for shaders
        // The struct must contain members that all implement UniformValue
    };
    output.into()
}
