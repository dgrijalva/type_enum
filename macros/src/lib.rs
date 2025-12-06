use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(TypeEnum)]
pub fn type_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match &input.data {
        Data::Enum(data) => data,
        _ => panic!("TypeEnum can only be derived for enums"),
    };

    let mut from_impls = Vec::new();
    let mut type_enum_impls = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Single field tuple variant like Number(i64)
                let field_type = &fields.unnamed[0].ty;

                // Generate From implementation
                from_impls.push(quote! {
                    impl From<#field_type> for #name {
                        fn from(value: #field_type) -> Self {
                            #name::#variant_name(value)
                        }
                    }
                });

                // Generate TypeEnum implementation
                type_enum_impls.push(quote! {
                    impl crate::TypeEnum<#field_type> for #name {
                        fn value(&self) -> Option<&#field_type> {
                            match self {
                                #name::#variant_name(ref val) => Some(val),
                                _ => None,
                            }
                        }

                        fn value_mut(&mut self) -> Option<&mut #field_type> {
                            match self {
                                #name::#variant_name(ref mut val) => Some(val),
                                _ => None,
                            }
                        }

                        fn into_value(self) -> Result<#field_type, Self> {
                            match self {
                                #name::#variant_name(val) => Ok(val),
                                other => Err(other),
                            }
                        }
                    }
                });
            }
            Fields::Unnamed(fields) if fields.unnamed.len() > 1 => {
                // Multiple field tuple variant like Tuple(u8, u8)
                let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                let tuple_type = quote! { (#(#field_types),*) };

                let field_indices: Vec<Index> =
                    (0..fields.unnamed.len()).map(Index::from).collect();

                // Generate From implementation
                from_impls.push(quote! {
                    impl From<#tuple_type> for #name {
                        fn from(value: #tuple_type) -> Self {
                            #name::#variant_name(#(value.#field_indices),*)
                        }
                    }
                });

                // Generate field destructuring patterns
                let field_names = (0..fields.unnamed.len())
                    .map(|i| {
                        syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site())
                    })
                    .collect::<Vec<_>>();

                // Generate TypeEnum implementation
                // Note: value() and value_mut() return None for multi-field variants
                // since we can't safely return references to reconstructed tuples
                type_enum_impls.push(quote! {
                    impl crate::TypeEnum<#tuple_type> for #name {
                        fn value(&self) -> Option<&#tuple_type> {
                            // Can't return reference to temporary tuple, so return None
                            None
                        }

                        fn value_mut(&mut self) -> Option<&mut #tuple_type> {
                            // Can't return reference to temporary tuple, so return None
                            None
                        }

                        fn into_value(self) -> Result<#tuple_type, Self> {
                            match self {
                                #name::#variant_name(#(#field_names),*) => Ok((#(#field_names),*)),
                                other => Err(other),
                            }
                        }
                    }
                });
            }
            Fields::Unnamed(_) => {
                panic!("Empty tuple variants are not supported");
            }
            _ => panic!(
                "Only tuple variants are supported (struct-style variants are not supported)"
            ),
        }
    }

    let expanded = quote! {
        #(#from_impls)*
        #(#type_enum_impls)*
    };

    TokenStream::from(expanded)
}
