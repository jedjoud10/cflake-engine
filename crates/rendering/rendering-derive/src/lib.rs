use proc_macro::{self, TokenStream};
use quote::{__private::Span, quote, quote_spanned};
use syn::{
    parse_macro_input,
    token::{self, Enum, Union},
    Data, DataEnum, DataStruct, DataUnion, DeriveInput, Error, Fields, LitStr, Result,
};

#[proc_macro_derive(Uniform)]
pub fn derive_components(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let res = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields), ..
        }) => {
            let data_expanded_members = fields.named.into_iter().map(|field| {
                let field_name = field.ident.expect("Unreachable");
                let span = field_name.span();
                let field_name_stringified = LitStr::new(&field_name.to_string(), span);
                quote_spanned! { span=>
                    self.#field_name.set_raw_uniform_value(#field_name_stringified, bound);
                }
            });

            let output = quote! {
                // Uniforms are created using the proper Uniform derive macro
                // The macro will take any struct, and use it as uniforms for shaders
                // The struct must contain members that all implement UniformValue
                unsafe impl #impl_generics UniformStruct for #ident #ty_generics #where_clause {
                    unsafe fn set_uniform_values(&self, bound: &mut Active<Program>) {
                        #(#data_expanded_members)*
                    }
                }
            };

            Ok(output)
        }
        Data::Enum(DataEnum { enum_token: Enum { span }, .. }) | Data::Union(DataUnion { union_token: Union { span }, .. }) => Err(Error::new(span, "Expected a `struct`")),

        Data::Struct(_) => Err(Error::new(Span::call_site(), "Expected a `struct` with named fields")),
    };

    TokenStream::from(match res {
        Ok(it) => it,
        Err(err) => err.to_compile_error(),
    })
}
