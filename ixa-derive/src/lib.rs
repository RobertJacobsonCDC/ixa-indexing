extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    // Expr,
    // ExprParen,
    // ExprTuple,
    Ident,
    Token,
    Type,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
};


#[proc_macro]
pub fn sorted_tuple_impl(input: TokenStream) -> TokenStream {
    // Parse the input as a list of types
    let types = parse_macro_input!(input with Punctuated::<Type, Token![,]>::parse_terminated);

    let original_types: Vec<_> = types.iter().cloned().collect();

    // Sort the types by their string representation
    let mut sorted_types = original_types.clone();
    sorted_types.sort_by(|a, b| quote!(#a).to_string().cmp(&quote!(#b).to_string()));

    // Create tuple binding names: t0, t1, ...
    let original_bindings: Vec<_> = (0..original_types.len())
        .map(|i| syn::Ident::new(&format!("t{}", i), proc_macro2::Span::call_site()))
        .collect();

    let sorted_bindings: Vec<_> = (0..sorted_types.len())
        .map(|i| syn::Ident::new(&format!("s{}", i), proc_macro2::Span::call_site()))
        .collect();

    // `to_sorted_tuple` expression: map sorted types back to original binding positions
    let to_sorted_exprs: Vec<_> = sorted_types.iter().map(|ty| {
        let idx = original_types
            .iter()
            .position(|t| quote!(#t).to_string() == quote!(#ty).to_string())
            .expect("type not found in original list");
        let ident = &original_bindings[idx];
        quote!(#ident)
    }).collect();

    // `from_sorted_tuple` expression: map original types to sorted binding positions
    let from_sorted_exprs: Vec<_> = original_types.iter().map(|ty| {
        let idx = sorted_types
            .iter()
            .position(|t| quote!(#t).to_string() == quote!(#ty).to_string())
            .expect("type not found in sorted list");
        let ident = &sorted_bindings[idx];
        quote!(#ident)
    }).collect();

    let tuple_type = quote! { ( #( #original_types ),* ) };
    let sorted_tuple_type = quote! { ( #( #sorted_types ),* ) };

    let expanded = quote! {
        impl SortableTuple for #tuple_type {
            type Sorted = #sorted_tuple_type;

            fn to_sorted_tuple(self) -> Self::Sorted {
                let ( #( #original_bindings ),* ) = self;
                ( #( #to_sorted_exprs ),* )
            }

            fn from_sorted_tuple(sorted: Self::Sorted) -> Self {
                let ( #( #sorted_bindings ),* ) = sorted;
                ( #( #from_sorted_exprs ),* )
            }
        }
    };

    TokenStream::from(expanded)
}


/// Struct to parse `sorted_tag_value_impl!(tag_tuple = (...), value_tuple = (...));`
struct TagValueInput {
    tag_tuple: Vec<Type>,
    value_tuple: Vec<Type>,
}

impl Parse for TagValueInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse `tag_tuple = ( ... ),`
        let tag_key: Ident = input.parse()?;
        if tag_key != "tag_tuple" {
            return Err(input.error("expected `tag_tuple`"));
        }
        input.parse::<Token![=]>()?;

        let tag_tuple;
        syn::parenthesized!(tag_tuple in input);
        let tags = tag_tuple.parse_terminated(Type::parse, Token![,])?;
        input.parse::<Token![,]>()?;

        // Parse `value_tuple = ( ... )`
        let value_key: Ident = input.parse()?;
        if value_key != "value_tuple" {
            return Err(input.error("expected `value_tuple`"));
        }
        input.parse::<Token![=]>()?;

        let value_tuple;
        syn::parenthesized!(value_tuple in input);
        let values = value_tuple.parse_terminated(Type::parse, Token![,])?;

        Ok(TagValueInput {
            tag_tuple: tags.into_iter().collect(),
            value_tuple: values.into_iter().collect(),
        })
    }
}


#[proc_macro]
pub fn sorted_tag_value_impl(input: TokenStream) -> TokenStream {
    let TagValueInput {
        tag_tuple,
        value_tuple,
    } = parse_macro_input!(input as TagValueInput);

    let orig_tags: Vec<_> = tag_tuple.into_iter().collect();
    let orig_values: Vec<_> = value_tuple.into_iter().collect();

    assert_eq!(orig_tags.len(), orig_values.len());

    // Compute sorted tags and permutation
    let mut indexed_tags: Vec<_> = orig_tags.iter().enumerate().collect();
    indexed_tags.sort_by_key(|(_, ty)| quote!(#ty).to_string());

    let sorted_tags: Vec<_> = indexed_tags.iter().map(|(_, t)| (*t).clone()).collect();
    let reordered_value_types: Vec<_> = indexed_tags.iter()
                                                    .map(|(i, _)| orig_values[*i].clone())
                                                    .collect();

    // Generate binding names
    let t_bindings: Vec<_> = (0..orig_values.len())
        .map(|i| Ident::new(&format!("t{}", i), proc_macro2::Span::call_site()))
        .collect();
    let s_bindings: Vec<_> = (0..orig_values.len())
        .map(|i| Ident::new(&format!("s{}", i), proc_macro2::Span::call_site()))
        .collect();

    // to_sorted: map sorted index to original binding
    let to_sorted_exprs: Vec<_> = indexed_tags
        .iter()
        .map(|(i, _)| {
            let ident = &t_bindings[*i];
            quote!(#ident)
        })
        .collect();

    // from_sorted: map original index to binding in sorted
    let from_sorted_exprs: Vec<_> = orig_tags
        .iter()
        .map(|ty| {
            let pos = sorted_tags
                .iter()
                .position(|t| quote!(#t).to_string() == quote!(#ty).to_string())
                .unwrap();
            let ident = &s_bindings[pos];
            quote!(#ident)
        })
        .collect();

    let tag_type = quote! { ( #( #orig_tags ),* ) };
    let value_type = quote! { ( #( #orig_values ),* ) };
    let sorted_tag_type = quote! { ( #( #sorted_tags ),* ) };
    let reordered_value_type = quote! { ( #( #reordered_value_types ),* ) };

    let expanded = quote! {
        impl SortByTag<#tag_type> for #value_type {
            type SortedTag = #sorted_tag_type;
            type ReorderedValue = #reordered_value_type;

            fn reorder_by_tag(self) -> Self::ReorderedValue {
                let ( #( #t_bindings ),* ) = self;
                ( #( #to_sorted_exprs ),* )
            }

            fn unreorder_by_tag(sorted: Self::ReorderedValue) -> Self {
                let ( #( #s_bindings ),* ) = sorted;
                ( #( #from_sorted_exprs ),* )
            }
        }
    };

    TokenStream::from(expanded)
}
