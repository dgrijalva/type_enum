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
    let mut trait_impls = Vec::new();

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

                // Generate Value implementation for &'a T
                trait_impls.push(quote! {
                    impl<'a> crate::Value<'a, &'a #field_type> for #name {
                        fn value(&'a self) -> Option<&'a #field_type> {
                            match self {
                                #name::#variant_name(ref val) => Some(val),
                                _ => None,
                            }
                        }
                    }
                });

                // Generate ValueMut implementation for &'a mut T
                trait_impls.push(quote! {
                    impl<'a> crate::ValueMut<'a, &'a mut #field_type> for #name {
                        fn value_mut(&'a mut self) -> Option<&'a mut #field_type> {
                            match self {
                                #name::#variant_name(ref mut val) => Some(val),
                                _ => None,
                            }
                        }
                    }
                });

                // Generate IntoValue implementation for T
                trait_impls.push(quote! {
                    impl crate::IntoValue<#field_type> for #name {
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

                // Generate field names for destructuring
                let field_names = (0..fields.unnamed.len())
                    .map(|i| {
                        syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site())
                    })
                    .collect::<Vec<_>>();

                // Generate Value implementation for (&'a T1, &'a T2, ...)
                let ref_tuple_type = quote! { (#(&'a #field_types),*) };
                trait_impls.push(quote! {
                    impl<'a> crate::Value<'a, #ref_tuple_type> for #name {
                        fn value(&'a self) -> Option<#ref_tuple_type> {
                            match self {
                                #name::#variant_name(#(ref #field_names),*) => Some((#(#field_names),*)),
                                _ => None,
                            }
                        }
                    }
                });

                // Generate ValueMut implementation for (&'a mut T1, &'a mut T2, ...)
                let mut_ref_tuple_type = quote! { (#(&'a mut #field_types),*) };
                trait_impls.push(quote! {
                    impl<'a> crate::ValueMut<'a, #mut_ref_tuple_type> for #name {
                        fn value_mut(&'a mut self) -> Option<#mut_ref_tuple_type> {
                            match self {
                                #name::#variant_name(#(ref mut #field_names),*) => Some((#(#field_names),*)),
                                _ => None,
                            }
                        }
                    }
                });

                // Generate IntoValue implementation for (T1, T2, ...)
                trait_impls.push(quote! {
                    impl crate::IntoValue<#tuple_type> for #name {
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
        #(#trait_impls)*
    };

    TokenStream::from(expanded)
}
