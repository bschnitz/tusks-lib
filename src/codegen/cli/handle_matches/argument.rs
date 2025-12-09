use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, parse_str};

use crate::Argument;

/// TODO: some things are handled quite inefficiently during runtime here. It is not really of
/// importance, since the impact is probably relativly low. But it would be interesting to think
/// about how to improve the code.
impl Argument {
    pub fn extract_from_matches(&self, matches_var: &Ident) -> TokenStream {
        let arg_name = &self.name;

        if self.flag {
            return quote! { #matches_var.subcommand().unwrap().1.get_flag(#arg_name) };
        }

        let type_token: syn::Type = parse_str(&self.type_)
            .expect(&format!("Invalid Rust type: {}", self.type_));

        let getter_str = match self.count {
            Some(_) => "get_many",
            None => "get_one"
        };
        let getter = Ident::new(getter_str, Span::call_site());

        if let Some(default) = &self.default {
            // Argument has a default value
            let collect_tokens = match self.count {
                Some(_) => quote! { .cloned().collect::<Vec<_>>().into() },
                None => quote! { .clone() }
            };
            
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .map(|v| v #collect_tokens)
                    .unwrap_or_else(|| #default.to_string().parse().unwrap() )
            }
        } else if self.optional {
            // Optional arguments
            let collect_tokens = match self.count {
                Some(_) => quote! { .cloned().collect::<Vec<_>>().into() },
                None => quote! { .clone() }
            };
            
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .map(|v| v #collect_tokens)
            }
        } else {
            // Required arguments
            let collect_tokens = match self.count {
                Some(_) => quote! { .cloned().collect::<Vec<_>>().into() },
                None => quote! { .clone() }
            };
            
            quote! {
                #matches_var.subcommand().unwrap().1
                    .#getter::<#type_token>(#arg_name)
                    .unwrap()
                    #collect_tokens
            }
        }
    }
}
