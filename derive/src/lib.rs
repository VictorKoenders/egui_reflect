#![warn(clippy::nursery, clippy::pedantic)]

extern crate proc_macro;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::{Attribute, Meta};

/// Derive macro for implementing `Reflectable` for a struct.
///
/// # Panics
///
/// Panics when:
/// - The `EguiReflect` derive is used on anything other than a struct.
/// - A struct field has no ident.
/// - A `range` attribute is not followed by a literal.
/// - A `range` attribute is not followed by a comma.
#[proc_macro_derive(EguiReflect, attributes(reflect))]
pub fn egui_reflect_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = input.ident;

    // Get the fields of the struct
    let fields = if let syn::Data::Struct(data) = input.data {
        data.fields
    } else {
        panic!("EguiReflect can only be derived for structs");
    };

    // Generate the reflect method
    let reflect_fields = fields.iter().filter_map(|field| {
        let attrs = FieldAttributes::from_attributes(&field.attrs);

        if attrs.skip {
            return None;
        }

        let opts = attrs.opts;
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let display_name = heck::AsTitleCase(field_name_str.as_str()).to_string();
        Some(quote! {
            egui_reflect::ReflectField {
                name: #display_name,
                value: &mut self.#field_name,
                opts: egui_reflect::FieldOptions {
                    #opts
                    ..Default::default()
                },
            }
        })
    });

    // Generate the final implementation
    let expanded = quote! {
        impl egui_reflect::Reflectable for #name {
            #[allow(clippy::needless_update)]
            fn reflect(&mut self) -> Vec<egui_reflect::ReflectField> {
                vec![
                    #(#reflect_fields),*
                ]
            }
        }
    };

    // Convert the expanded code into a TokenStream and return it
    proc_macro::TokenStream::from(expanded)
}

struct FieldAttributes {
    skip: bool,
    opts: TokenStream,
}

impl FieldAttributes {
    fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut skip = false;
        let mut opts = TokenStream::new();

        for attr in attrs {
            if let Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("reflect") {
                    let mut token_iter = meta_list.tokens.clone().into_iter();
                    while let Some(token) = token_iter.next() {
                        if let TokenTree::Ident(ident) = token {
                            if ident == "skip" {
                                skip = true;
                            }
                            if ident == "range" {
                                let Some(TokenTree::Punct(p)) = token_iter.next() else {
                                    panic!("Expected range to be followed by a punct");
                                };
                                assert!(
                                    p.as_char() == '=',
                                    "Expected range to be followed by an equals sign"
                                );
                                let Some(TokenTree::Literal(min)) = token_iter.next() else {
                                    panic!("Expected range to be followed by a literal");
                                };
                                let Some(TokenTree::Punct(p)) = token_iter.next() else {
                                    panic!("Expected range to be followed by a punct");
                                };
                                assert!(
                                    p.as_char() == ',',
                                    "Expected range to be followed by a period"
                                );
                                let Some(TokenTree::Literal(max)) = token_iter.next() else {
                                    panic!("Expected range to be followed by a literal");
                                };

                                opts = quote! {
                                    range: Some((#min, #max)),
                                };
                            }
                        }
                    }
                }
            }
        }

        Self { skip, opts }
    }
}
