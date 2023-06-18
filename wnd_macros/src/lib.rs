#![allow(unused_imports)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{*, spanned::Spanned, Result};

#[cfg(feature = "todo_attr")]
#[proc_macro_attribute]
pub fn todo_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let args = parse_macro_input!(attr as AttributeArgs);

    // Extract the message from the attribute arguments
    let message = if !args.is_empty() {
        let first_arg = &args[0];
        if let syn::NestedMeta::Lit(lit) = first_arg {
            lit.to_token_stream().to_string()
        } else {
            return syn::Error::new_spanned(first_arg, "Invalid attribute argument").to_compile_error().into();
        }
    } else {
        String::new()
    };

    // Parse the annotated function
    let mut func = parse_macro_input!(input as ItemFn);

    // Add #[allow(unused)] attribute to the function
    let allow_unused_attr = syn::parse_quote! { #[allow(unused)] };
    func.attrs.push(allow_unused_attr);

    // Replace the function body with `todo!()` macro and the message
    let todo_macro = if message.is_empty() {
        quote! { todo!() }
    } else {
        quote! { todo!(#message) }
    };
    func.block = Box::new(syn::parse_quote! { { #todo_macro } });

    // Return the modified function code as a TokenStream
    TokenStream::from(quote! { #func })
}

#[cfg(feature = "thread")]
#[proc_macro_attribute]
pub fn thread(attr: TokenStream, input: TokenStream) -> TokenStream {
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
    quote! {#input}.into()
}
