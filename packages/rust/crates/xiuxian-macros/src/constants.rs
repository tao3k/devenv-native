//! Constant-generating procedural macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub(crate) fn expand_patterns(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    );

    let mut expanded = Vec::with_capacity(items.len());
    for expr in items {
        match expr {
            Expr::Tuple(tuple) if tuple.elems.len() == 2 => {
                let name = &tuple.elems[0];
                let pattern = &tuple.elems[1];
                expanded.push(quote! {
                    pub const #name: &str = #pattern;
                });
            }
            Expr::Tuple(tuple) => {
                return syn::Error::new_spanned(
                    tuple,
                    "patterns! requires tuple of (NAME, pattern_string)",
                )
                .to_compile_error()
                .into();
            }
            other => {
                return syn::Error::new_spanned(
                    other,
                    "patterns! requires tuple of (NAME, pattern_string)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    quote! {
        #(#expanded)*
    }
    .into()
}

pub(crate) fn expand_topics(input: TokenStream) -> TokenStream {
    let items = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    );

    let mut expanded = Vec::with_capacity(items.len());
    for expr in items {
        match expr {
            Expr::Tuple(tuple) if tuple.elems.len() == 2 => {
                let name = &tuple.elems[0];
                let value = &tuple.elems[1];
                expanded.push(quote! {
                    pub const #name: &str = #value;
                });
            }
            Expr::Tuple(tuple) => {
                return syn::Error::new_spanned(
                    tuple,
                    "topics! requires tuple of (CONST_NAME, string_value)",
                )
                .to_compile_error()
                .into();
            }
            other => {
                return syn::Error::new_spanned(
                    other,
                    "topics! requires tuple of (CONST_NAME, string_value)",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    quote! {
        #(#expanded)*
    }
    .into()
}
