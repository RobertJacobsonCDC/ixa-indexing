extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Token, Type};

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
