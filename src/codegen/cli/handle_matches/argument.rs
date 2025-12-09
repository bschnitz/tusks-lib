use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, parse_str};

use crate::Argument;

impl Argument {
    pub fn extract_from_matches(&self, matches_var: &Ident) -> TokenStream {
        let arg_name = &self.name;

        let type_token: syn::Type = parse_str(&self.type_)
            .expect(&format!("Invalid Rust type: {}", self.type_));

        let getter_str = match self.count {
            Some(_) => "get_many",
            None => "get_one"
        };
        let getter = Ident::new(getter_str, Span::call_site());

        let collect_tokens = match self.count {
            Some(_) => quote! { .copied().collect::<Vec<_>>().into() },
            None => quote! {}
        };

        if self.flag {
            // Flags are booleans
            quote! { #matches_var.subcommand().unwrap().1.get_flag(#arg_name) }
        } else if let Some(default) = &self.default {
            // Argument has a default value
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| #default.to_string())
                    #collect_tokens
            }
        } else if self.optional {
            // Optional arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .map(|s| s.clone())
                    #collect_tokens
            }
        } else {
            // Required arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .unwrap()
                    .clone()
                    #collect_tokens
            }
        }
    }
}
