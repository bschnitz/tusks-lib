use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::TusksNode;
use crate::codegen::cli::handle_matches::context::BuildContext;

impl TusksNode {
    pub fn build_handle_matches(&self, path_sep: &str) -> TokenStream {
        // Generated variables for the function
        let matches_var = Ident::new("matches", Span::call_site());
        let link_path_var = Ident::new("link_path", Span::call_site());

        let ctx = BuildContext{
            matches_var: &matches_var,
            link_path_var: &link_path_var,
            path_sep,
        };

        // Collect all match arms for Tusks
        let match_arms: Vec<TokenStream> = self.iter_all_tusks()
            .map(|(node, tusk)| tusk.build_match_arm(node, &ctx))
            .collect();

        // Collect all link calls for fallback
        let link_handle_matches: Vec<TokenStream> = self.iter_all_links()
            .map(|(parent, link)| link.build_link_handle_matches(parent, &ctx))
            .collect();

        quote! {
            pub fn handle_matches(#matches_var: &clap::ArgMatches, #link_path_var: Vec<String>) -> bool {
                let (subcommand_name, sub_matches) = #matches_var.subcommand().unwrap();
                let subcommand_prefix = #link_path_var.join(#path_sep);

                // Return early if the command does not start with our prefix
                if !subcommand_prefix.is_empty() && !subcommand_name.starts_with(&subcommand_prefix) {
                    return false;
                }

                // Remove prefix from command name
                let actual_command = if !subcommand_prefix.is_empty() {
                    let prefix_len = subcommand_prefix.len() + #path_sep.len();
                    &subcommand_name[prefix_len..]
                } else {
                    subcommand_name
                };

                match actual_command {
                    #(#match_arms)*
                    _ => {
                        #(#link_handle_matches)*
                        eprintln!("Unknown command: {}", subcommand_name);
                        return true;
                    }
                }
            }
        }
    }
}
