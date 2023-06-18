#![allow(unused_imports)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{*, spanned::Spanned, Result};

#[cfg(feature = "todo_attr")]
#[proc_macro_attribute]
pub fn todo_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    // No need to parse the attribute arguments, let `todo!` handle that.
    let message: TokenStream2 = args.into();

    // Parse the annotated function
    let mut func = parse_macro_input!(input as ItemFn);

    // Add #[allow(unused)] attribute to the function
    let allow_unused_attr = syn::parse_quote! { #[allow(unused)] };
    func.attrs.push(allow_unused_attr);

    // Replace the function body with `todo!()` macro and the message
    let todo_macro = quote! {
        todo!(#message)
    };
    func.block = Box::new(syn::parse_quote! { { #todo_macro } });

    // Return the modified function code as a TokenStream
    TokenStream::from(debug_output(quote! { #func }))
}

#[cfg(feature = "thread")]
#[proc_macro_attribute]
pub fn thread(args: TokenStream, input: TokenStream) -> TokenStream {
    let _: parse::Nothing = parse_macro_input!(args);
    let mut input: syn::ItemFn = syn::parse_macro_input!(input);
    let fb = input.block;
    let rv = match input.sig.output.clone() {
        syn::ReturnType::Default => {
            syn::Type::Tuple(syn::parse_quote!{()})
        },
        syn::ReturnType::Type(_, t) => {
            *t
        }
    };
    let rv = quote::quote_spanned! { rv.span() => #rv };
    input.block = syn::parse2(quote! {{
        ::std::thread::spawn(move || {#fb})
    }}).unwrap();
    input.sig.output = syn::ReturnType::Type(
        Token![->](input.sig.output.span()),
        Box::new(syn::parse_quote! { ::std::thread::JoinHandle<#rv> }),
    );
    debug_output(quote! {#input}).into()
}

// ADDED BY ME:
fn debug_output(ts: TokenStream2) -> TokenStream2 {
    if ::std::env::var("DEBUG_EXPANSIONS").ok().map_or(true, |s| s != "1") {
        return ts;
    }

    let random: u64 = {
        use ::std::hash::{BuildHasher, Hasher};

        ::std::collections::hash_map::RandomState::new().build_hasher().finish()
    };
    let file_name = &format!("/tmp/{:016x}.rs", random);

    ::std::fs::write(file_name, ts.to_string()).unwrap();
    ::std::process::Command::new("rustfmt").args([
        "--edition", "2021", file_name,
    ]).status().unwrap();

    quote!(
        const _: () = {
            #[deprecated = ::core::concat!(
                "\n<DEBUG> output generated at ", #file_name,
            )]
            struct _Debug {}
            let _: _Debug;
        };

        ::core::include!(#file_name);
    )
}
