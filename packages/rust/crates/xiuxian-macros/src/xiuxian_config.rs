use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Expr, Lit, MetaNameValue, Result as SynResult, Token, parse_macro_input};

struct XiuxianConfigArgs {
    namespace: String,
    internal_path: Option<String>,
    orphan_file: Option<String>,
    array_merge: Option<String>,
}

impl Parse for XiuxianConfigArgs {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let mut namespace: Option<String> = None;
        let mut internal_path: Option<String> = None;
        let mut orphan_file: Option<String> = None;
        let mut array_merge: Option<String> = None;

        while !input.is_empty() {
            let meta: MetaNameValue = input.parse()?;
            let Some(ident) = meta.path.get_ident() else {
                return Err(Error::new_spanned(meta.path, "expected identifier key"));
            };
            let value = parse_string_literal(meta.value)?;
            match ident.to_string().as_str() {
                "namespace" => namespace = Some(value),
                "internal_path" => internal_path = Some(value),
                "orphan_file" => orphan_file = Some(value),
                "array_merge" => array_merge = Some(value),
                _ => {
                    return Err(Error::new_spanned(
                        ident,
                        "unsupported xiuxian_config argument; expected `namespace`, `internal_path`, `orphan_file`, or `array_merge`",
                    ));
                }
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        let Some(namespace) = namespace else {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "missing required argument `namespace = \"...\"`",
            ));
        };

        Ok(Self {
            namespace,
            internal_path,
            orphan_file,
            array_merge,
        })
    }
}

fn parse_string_literal(expr: Expr) -> SynResult<String> {
    match expr {
        Expr::Lit(expr_lit) => match expr_lit.lit {
            Lit::Str(value) => Ok(value.value()),
            _ => Err(Error::new_spanned(
                expr_lit,
                "expected string literal value",
            )),
        },
        other => Err(Error::new_spanned(other, "expected string literal value")),
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as XiuxianConfigArgs);
    let input_struct = parse_macro_input!(item as syn::ItemStruct);
    let struct_ident = &input_struct.ident;

    let namespace = args.namespace;
    let internal_path = args
        .internal_path
        .unwrap_or_else(|| format!("resources/config/{namespace}.toml"));
    let orphan_file = args
        .orphan_file
        .unwrap_or_else(|| format!("{namespace}.toml"));
    let array_merge = args.array_merge.unwrap_or_else(|| "overwrite".to_string());
    let array_merge_strategy = match array_merge.as_str() {
        "overwrite" => quote!(xiuxian_config_core::ArrayMergeStrategy::Overwrite),
        "append" => quote!(xiuxian_config_core::ArrayMergeStrategy::Append),
        _ => {
            return Error::new(
                proc_macro2::Span::call_site(),
                "invalid `array_merge`; expected \"overwrite\" or \"append\"",
            )
            .to_compile_error()
            .into();
        }
    };

    quote! {
        #input_struct

        impl #struct_ident {
            fn __xiuxian_config_namespace() -> &'static str {
                #namespace
            }

            fn __xiuxian_config_embedded_toml() -> &'static str {
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #internal_path))
            }

            fn __xiuxian_config_spec() -> xiuxian_config_core::ConfigCascadeSpec<'static> {
                xiuxian_config_core::ConfigCascadeSpec::new(
                    Self::__xiuxian_config_namespace(),
                    Self::__xiuxian_config_embedded_toml(),
                    #orphan_file
                )
                .with_embedded_source_path(concat!(env!("CARGO_MANIFEST_DIR"), "/", #internal_path))
                .with_array_merge_strategy(#array_merge_strategy)
            }

            /// Return merged TOML value from embedded defaults and cascading overrides.
            ///
            /// # Errors
            ///
            /// Returns an error when embedded/default TOML is invalid, when
            /// conflict enforcement fails, or when one override file cannot be
            /// parsed.
            pub(crate) fn __xiuxian_config_merged_value() -> Result<toml::Value, String> {
                xiuxian_config_core::resolve_and_merge_toml(Self::__xiuxian_config_spec())
                    .map_err(|error| error.to_string())
            }

            /// Return merged TOML value from embedded defaults and cascading overrides
            /// with explicit path roots.
            ///
            /// # Errors
            ///
            /// Returns an error when embedded/default TOML is invalid, when
            /// conflict enforcement fails, or when one override file cannot be
            /// parsed.
            pub(crate) fn __xiuxian_config_merged_value_with_paths(
                project_root: Option<&std::path::Path>,
                config_home: Option<&std::path::Path>,
            ) -> Result<toml::Value, String> {
                xiuxian_config_core::resolve_and_merge_toml_with_paths(
                    Self::__xiuxian_config_spec(),
                    project_root,
                    config_home,
                )
                .map_err(|error| error.to_string())
            }

            /// Load configuration from embedded defaults and cascading overrides.
            ///
            /// # Errors
            ///
            /// Returns an error when embedded/default TOML is invalid, when
            /// conflict enforcement fails, or when merged TOML cannot deserialize
            /// into the target config struct.
            pub fn load() -> Result<Self, String>
            where
                Self: serde::de::DeserializeOwned,
            {
                xiuxian_config_core::resolve_and_load(Self::__xiuxian_config_spec())
                    .map_err(|error| error.to_string())
            }

            /// Load configuration from embedded defaults and cascading overrides
            /// with explicit path roots.
            ///
            /// # Errors
            ///
            /// Returns an error when embedded/default TOML is invalid, when
            /// conflict enforcement fails, or when merged TOML cannot deserialize
            /// into the target config struct.
            pub fn load_with_paths(
                project_root: Option<&std::path::Path>,
                config_home: Option<&std::path::Path>,
            ) -> Result<Self, String>
            where
                Self: serde::de::DeserializeOwned,
            {
                xiuxian_config_core::resolve_and_load_with_paths(
                    Self::__xiuxian_config_spec(),
                    project_root,
                    config_home,
                )
                .map_err(|error| error.to_string())
            }
        }
    }
    .into()
}
