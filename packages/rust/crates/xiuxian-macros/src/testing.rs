//! Test helper procedural macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub(crate) fn expand_temp_dir(_input: TokenStream) -> TokenStream {
    quote! {
        {
            let path = std::env::temp_dir()
                .join(format!("omni_test_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path)
                .expect("Failed to create temp directory");
            path
        }
    }
    .into()
}

pub(crate) fn expand_assert_timing(input: TokenStream) -> TokenStream {
    let items: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if items.len() != 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "assert_timing! requires 2 arguments: (max_ms, block)",
        )
        .to_compile_error()
        .into();
    }

    let max_ms = &items[0];
    let block = &items[1];

    quote! {
        {
            let start = std::time::Instant::now();
            #block
            let elapsed = start.elapsed();
            let ms = elapsed.as_secs_f64() * 1000.0;
            assert!(
                ms < #max_ms,
                "Operation took {:.2}ms, expected < {}ms",
                ms,
                #max_ms
            );
            elapsed
        }
    }
    .into()
}

pub(crate) fn expand_bench_case(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as syn::Expr);

    quote! {
        {
            let start = std::time::Instant::now();
            let _ = #block;
            start.elapsed()
        }
    }
    .into()
}
