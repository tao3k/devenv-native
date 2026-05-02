//! Environment and string-selection procedural macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

pub(crate) fn expand_non_empty(input: TokenStream) -> TokenStream {
    let args: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if args.len() != 1 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "env_non_empty! requires exactly 1 argument: (env_var_name)",
        )
        .to_compile_error()
        .into();
    }

    let env_key_expr = &args[0];
    quote! {
        std::env::var(#env_key_expr)
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty())
    }
    .into()
}

pub(crate) fn expand_first_non_empty(input: TokenStream) -> TokenStream {
    let candidates: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if candidates.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "string_first_non_empty! requires at least one candidate",
        )
        .to_compile_error()
        .into();
    }

    quote! {
        {
            let mut resolved: Option<String> = None;
            for candidate in [#(#candidates),*] {
                if let Some(raw) = candidate {
                    let trimmed = raw.trim();
                    if !trimmed.is_empty() {
                        resolved = Some(trimmed.to_string());
                        break;
                    }
                }
            }
            resolved.unwrap_or_default()
        }
    }
    .into()
}
