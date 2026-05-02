//! PyO3-oriented procedural macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

pub(crate) fn expand_from(input: TokenStream) -> TokenStream {
    let items: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if items.len() != 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "py_from! requires exactly 2 arguments: (PyType, InnerType)",
        )
        .to_compile_error()
        .into();
    }

    let py_type = &items[0];
    let inner_type = &items[1];

    quote! {
        impl From<#inner_type> for #py_type {
            fn from(inner: #inner_type) -> Self {
                Self { inner }
            }
        }
    }
    .into()
}
