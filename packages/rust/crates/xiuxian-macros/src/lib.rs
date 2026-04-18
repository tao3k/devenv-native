//! # xiuxian-macros
//!
//! Common procedural macros for omni Rust crates.
//!
//! ## Macros
//!
//! ### Code Generation
//! - [`patterns!`] - Generate pattern constants for symbol extraction
//! - [`topics!`] - Generate topic/event constants
//! - [`py_from!`] - Generate `PyO3` From implementations
//! - [`env_non_empty!`] - Read a trimmed non-empty environment variable as `Option<String>`
//! - [`string_first_non_empty!`] - Resolve the first non-empty string candidate
//! - [`project_config_paths!`] - Build system/user/env layered config candidate paths
//! - [`crate_resources_dir!`] - Embed the calling crate's local `resources/` tree
//!
//! ### Testing Utilities
//! - [`temp_dir!`] - Create a temporary directory for tests
//! - [`assert_timing!`] - Assert timing constraint for benchmarks
//! - [`bench_case!`] - Create a benchmark test case

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

mod zhenfa_tool;

mod xiuxian_config;

/// Attribute macro for defining native zhenfa tools.
#[proc_macro_attribute]
pub fn zhenfa_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    zhenfa_tool::expand(attr, item)
}

/// Attribute macro for loading cascading config into a struct.
#[proc_macro_attribute]
pub fn xiuxian_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    xiuxian_config::expand(attr, item)
}

/// Generate pattern constants for symbol extraction.
#[proc_macro]
pub fn patterns(input: TokenStream) -> TokenStream {
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

/// Generate topic/event constants.
#[proc_macro]
pub fn topics(input: TokenStream) -> TokenStream {
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

/// Generate `PyO3` From implementations for wrapper types.
#[proc_macro]
pub fn py_from(input: TokenStream) -> TokenStream {
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

/// Read an environment variable and return `Option<String>` when non-empty after trim.
///
/// Input: `env_non_empty!("OPENAI_API_KEY")` or `env_non_empty!(dynamic_key_expr)`
#[proc_macro]
pub fn env_non_empty(input: TokenStream) -> TokenStream {
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

/// Resolve the first non-empty string from ordered `Option<&str>`-like candidates.
///
/// Input: `string_first_non_empty!(candidate_a, candidate_b, Some("fallback"))`
#[proc_macro]
pub fn string_first_non_empty(input: TokenStream) -> TokenStream {
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

/// Embed the calling crate's local `resources/` directory.
///
/// # Panics
///
/// Panics when `CARGO_MANIFEST_DIR` is unavailable while expanding the macro.
#[proc_macro]
pub fn crate_resources_dir(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
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

/// Build layered config candidate paths for a config filename.
///
/// Input: `project_config_paths!("qianji.toml", "QIANJI_CONFIG_PATH")`
///
/// Expansion order:
/// 1. `<PRJ_ROOT>/packages/conf/<file>`
/// 2. `<PRJ_CONFIG_HOME>/xiuxian-artisan-workshop/<file>` (`.config` when unset)
/// 3. `$<explicit_env>` when set and non-empty
#[proc_macro]
pub fn project_config_paths(input: TokenStream) -> TokenStream {
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

// ============================================================================
// Testing Utilities
// ============================================================================

/// Create a temporary directory for tests.
///
/// # Example
///
/// ```rust
/// let temp_path = xiuxian_macros::temp_dir!();
/// std::fs::write(temp_path.join("test.txt"), "hello")
///     .expect("temporary write should succeed");
/// assert!(temp_path.exists());
/// ```
#[proc_macro]
pub fn temp_dir(_input: TokenStream) -> TokenStream {
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

/// Assert timing constraint for benchmarks.
///
/// # Example
///
/// ```rust
/// let _elapsed = xiuxian_macros::assert_timing!(100.0, {
///     std::thread::sleep(std::time::Duration::from_millis(1));
/// });
/// ```
#[proc_macro]
pub fn assert_timing(input: TokenStream) -> TokenStream {
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

/// Create a benchmark test case with timing.
///
/// # Example
///
/// ```rust
/// let elapsed = xiuxian_macros::bench_case!(|| {
///     let value = 1 + 1;
///     assert_eq!(value, 2);
/// });
/// let _ = elapsed;
/// ```
#[proc_macro]
pub fn bench_case(input: TokenStream) -> TokenStream {
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
