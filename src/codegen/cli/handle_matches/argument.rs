use proc_macro2::{TokenStream};
use quote::quote;
use syn::Ident;

use crate::Argument;

impl Argument {
    pub fn extract_from_matches(&self, matches_var: &Ident) -> TokenStream {
        let arg_name = &self.name;

        if self.flag {
            // Flags are booleans
            quote! { #matches_var.subcommand().unwrap().1.get_flag(#arg_name) }
        } else if let Some(default) = &self.default {
            // Argument has a default value
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<String>(#arg_name)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| #default.to_string())
            }
        } else if self.optional {
            // Optional arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<String>(#arg_name)
                    .map(|s| s.clone())
            }
        } else {
            // Required arguments
            quote! {
                #matches_var.subcommand().unwrap().1
                    .get_one::<String>(#arg_name)
                    .unwrap()
                    .clone()
            }
        }
    }
}
