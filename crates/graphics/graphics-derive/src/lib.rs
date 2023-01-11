use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit, MetaList, NestedMeta};
use proc_macro_crate::{crate_name, FoundCrate};

#[proc_macro_derive(Bindings, attributes(sampler, frequency, view, ubo, push_constants))]
pub fn derive_descriptor_set(input: TokenStream) -> TokenStream {
    let found_crate = crate_name("graphics").expect("my-crate is present in `Cargo.toml`");

    let cratename = match found_crate {
        FoundCrate::Itself => {
            let ident = Ident::new("crate", Span::call_site());
            quote!( #ident )
        },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        } 
    };

    let DeriveInput { attrs,
        vis,
        ident,
        generics,
        data
    } = parse_macro_input!(input as DeriveInput);

    let mut vec = Vec::<proc_macro2::TokenStream>::new();
    handle_structure(data, &mut vec);

    let output = quote! {
        /*
        impl<'a> DescriptorSetLayout for #ident<'a> {
            fn descriptors() -> Vec<crate::BindingLayout> {
                todo!()
            }
        }
        impl<'a> DescriptorSet<'a> for #ident<'a> {}
        */
    };

    output.into()
}

fn handle_structure(data: syn::Data, vec: &mut Vec<proc_macro2::TokenStream>)  {
    match data {
        syn::Data::Struct(dstruct) => {
            match dstruct.fields {
                syn::Fields::Named(fields) => {
                    handle_named_fields(fields, vec);
                },
                _ => panic!("Structs with unnamed fields not supported"),
            }
        },
        _ => panic!("Material bindings only supported for structs")
    }
}

fn handle_named_fields(fields: syn::FieldsNamed, mut vec: &mut Vec<proc_macro2::TokenStream>)  {
    for field in fields.named.into_iter() {
        handle_filed(field, &mut vec);
    }
}

/*
	#[sampler(binding = 0)]		
	#[frequency(mesh)]
    #[view(fragment)]
*/

fn handle_filed(field: syn::Field, vec: &mut Vec<proc_macro2::TokenStream>) {  
    for attribute in field.attrs {
        let meta = attribute.parse_meta().unwrap();

        vec.push({
            quote! {

            }
        });

        match meta {
            syn::Meta::List(list) => {
                let name = list.path.get_ident().unwrap().to_string();

                match name.as_str() {
                    "sampler" => handle_sampler(list),
                    "ubo" => handle_ubo(list),
                    "frequency" => handle_frequency(list),
                    "view" => handle_view(list),
                    _ => panic!("Unknown attribute"),
                };
            }

            syn::Meta::Path(path) => {
                let name = path.get_ident().unwrap().to_string();

                match name.as_str() {
                    "push_constants" => handle_push_constant(),
                    _ => panic!("Unknown attribute"),
                }
            }

            _ => {}
        }
    }
}

fn handle_view(list: MetaList) {
    if list.nested.empty_or_trailing() {
        panic!("Missing view specifier(s)");
    }

    for nested_meta in list.nested.iter() {
        if let NestedMeta::Meta(meta) = nested_meta {
            match meta.path().get_ident().unwrap().to_string().as_str() {
                "fragment" => {},
                "vertex" => {},
                x => panic!("View specifier '{x}' not supported"),
            }
        }
    }
}

fn handle_frequency(list: MetaList) {
    let first = list.nested.iter().next().expect("Missing frequency specifier");
    if let NestedMeta::Meta(meta) = first {
        let frequency = meta.path().get_ident().unwrap().to_string();
        match frequency.as_str() {
            "global" => {},
            "instance" => {},
            "mesh" => {},
            x => panic!("Frequency specifier '{x}' not supported"),
        }
    }
}

fn handle_push_constant() {
}

fn handle_ubo(list: MetaList) {
}

fn handle_sampler(list: MetaList) {
}