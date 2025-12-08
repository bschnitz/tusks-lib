use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Builds a TokenStream representing a Rust path from any iterable of items convertible to &str.
/// Example: ["foo", "bar", "baz"] or [String::from("foo"), "bar", "baz"] -> `foo::bar::baz`
pub fn build_path_tokens<I, S>(segments: I) -> TokenStream
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut ts = TokenStream::new();
    let mut iter = segments.into_iter();

    if let Some(first) = iter.next() {
        let ident = Ident::new(first.as_ref(), Span::call_site());
        ts.extend(quote! { #ident });
    }

    for segment in iter {
        let ident = Ident::new(segment.as_ref(), Span::call_site());
        ts.extend(quote! { :: #ident });
    }

    ts
}
