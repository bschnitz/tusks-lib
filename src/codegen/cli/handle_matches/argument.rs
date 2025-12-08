use proc_macro2::{TokenStream};
use quote::quote;
use syn::{Ident, parse_str};

use crate::Argument;

impl Argument {
    pub fn extract_from_matches(&self, matches_var: &Ident) -> TokenStream {
        let arg_name = &self.name;

        let type_token: syn::Type = parse_str(&self.type_)
            .expect(&format!("Invalid Rust type: {}", self.type_));

        if self.flag {
            // Flags are booleans
            quote! { #matches_var.subcommand().unwrap().1.get_flag(#arg_name) }
        } else if let Some(default) = &self.default {
            // Argument has a default value
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<#type_token>(#arg_name)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| #default.to_string())
            }
        } else if self.optional {
            // Optional arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<#type_token>(#arg_name)
                    .map(|s| s.clone())
            }
        } else {
            // Required arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<#type_token>(#arg_name)
                    .unwrap()
                    .clone()
            }
        }
    }
}
