use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, Index, Meta, parse_macro_input};

/// Check if a variant has the #[type_enum(skip)] attribute
fn has_skip_attribute(variant: &syn::Variant) -> bool {
    for attr in &variant.attrs {
        if attr.path().is_ident("type_enum") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                if tokens == "skip" {
                    return true;
                }
            }
        }
    }
    false
}

/// Get a canonical string representation of a type for duplicate detection
fn type_key(fields: &Fields) -> String {
    match fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => quote!(#fields).to_string(),
        Fields::Unnamed(fields) => {
            let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
            quote!((#(#field_types),*)).to_string()
        }
        _ => String::new(),
    }
}

#[proc_macro_derive(TypeEnum, attributes(type_enum))]
pub fn type_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match &input.data {
        Data::Enum(data) => data,
        _ => panic!("TypeEnum can only be derived for enums"),
    };

    // First pass: collect types and check for duplicates (excluding skipped variants)
    let mut seen_types: HashMap<String, &syn::Variant> = HashMap::new();
    for variant in &data.variants {
        if has_skip_attribute(variant) {
            continue;
        }

        let key = type_key(&variant.fields);
        if !key.is_empty() {
            if let Some(first_variant) = seen_types.get(&key) {
                let first_name = &first_variant.ident;
                let second_name = &variant.ident;
                return syn::Error::new_spanned(
                    variant,
                    format!(
                        "duplicate type in enum: variants `{}` and `{}` both hold the same type(s). \
                        Each variant must hold a unique type. Use #[type_enum(skip)] to exclude a variant.",
                        first_name, second_name
                    ),
                )
                .to_compile_error()
                .into();
            }
            seen_types.insert(key, variant);
        }
    }

    let mut from_impls = Vec::new();
    let mut trait_impls = Vec::new();

    for variant in &data.variants {
        // Skip variants with #[type_enum(skip)] attribute
        if has_skip_attribute(variant) {
            continue;
        }

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
                                #name::#variant_name(val) => Some(val),
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
                                #name::#variant_name(val) => Some(val),
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
                                #name::#variant_name(#(#field_names),*) => Some((#(#field_names),*)),
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
                                #name::#variant_name(#(#field_names),*) => Some((#(#field_names),*)),
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
