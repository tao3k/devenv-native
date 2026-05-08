//! Resource and project-path procedural macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, parse_macro_input};

pub(crate) fn expand_crate_dir(input: TokenStream) -> TokenStream {
    if input.into_iter().next().is_some() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "crate_resources_dir! takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|error| panic!("failed to resolve CARGO_MANIFEST_DIR: {error}"));
    let resources_dir = std::path::Path::new(&manifest_dir).join("resources");
    let dir_literal = syn::LitStr::new(
        resources_dir.to_string_lossy().as_ref(),
        proc_macro2::Span::call_site(),
    );

    quote! {
        ::include_dir::include_dir!(#dir_literal)
    }
    .into()
}

pub(crate) fn expand_project_config_paths(input: TokenStream) -> TokenStream {
    let args: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if args.len() != 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "project_config_paths! requires exactly 2 string arguments: (file_name, explicit_env_var)",
        )
        .to_compile_error()
        .into();
    }

    let file_name = match &args[0] {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(value) => value,
            _ => {
                return syn::Error::new_spanned(
                    &args[0],
                    "first argument must be a string literal filename",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &args[0],
                "first argument must be a string literal filename",
            )
            .to_compile_error()
            .into();
        }
    };
    let explicit_env_var = match &args[1] {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(value) => value,
            _ => {
                return syn::Error::new_spanned(
                    &args[1],
                    "second argument must be a string literal env var name",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &args[1],
                "second argument must be a string literal env var name",
            )
            .to_compile_error()
            .into();
        }
    };

    quote! {
        {
            let project_root = if let Ok(raw) = std::env::var("PRJ_ROOT") {
                std::path::PathBuf::from(raw)
            } else {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            };

            let config_home = if let Ok(raw) = std::env::var("PRJ_CONFIG_HOME") {
                let path = std::path::PathBuf::from(raw);
                if path.is_absolute() {
                    path
                } else {
                    project_root.join(path)
                }
            } else {
                project_root.join(".config")
            };

            let mut candidates = vec![
                project_root.join(concat!("packages/conf/", #file_name)),
                config_home.join(concat!("xiuxian-artisan-workshop/", #file_name)),
            ];

            if let Ok(raw) = std::env::var(#explicit_env_var) {
                let explicit = raw.trim();
                if !explicit.is_empty() {
                    candidates.push(std::path::PathBuf::from(explicit));
                }
            }

            candidates
        }
    }
    .into()
}
