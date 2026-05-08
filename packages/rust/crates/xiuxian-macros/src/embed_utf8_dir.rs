use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, parse_macro_input};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args: Vec<Expr> = parse_macro_input!(
        input with syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
    )
    .into_iter()
    .collect();

    if args.len() != 1 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "embed_utf8_dir! requires exactly 1 argument: (directory_literal)",
        )
        .to_compile_error()
        .into();
    }

    let dir_literal = &args[0];
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let const_ident = format_ident!("__OMNI_EMBEDDED_DIR_{}", suffix);
    let collect_fn_ident = format_ident!("__omni_collect_utf8_files_{}", suffix);
    let vec_ident = format_ident!("__omni_embedded_files_{}", suffix);

    quote! {
        {
            const #const_ident: ::include_dir::Dir<'_> = ::include_dir::include_dir!(#dir_literal);

            fn #collect_fn_ident(
                dir: &::include_dir::Dir<'_>,
                out: &mut Vec<(String, String)>,
            ) {
                for file in dir.files() {
                    if let Some(content) = file.contents_utf8() {
                        out.push((
                            file.path().to_string_lossy().replace('\\', "/"),
                            content.to_string(),
                        ));
                    }
                }
                for child in dir.dirs() {
                    #collect_fn_ident(child, out);
                }
            }

            let mut #vec_ident = Vec::new();
            #collect_fn_ident(&#const_ident, &mut #vec_ident);
            #vec_ident.sort_by(|left, right| left.0.cmp(&right.0));
            #vec_ident
        }
    }
    .into()
}
