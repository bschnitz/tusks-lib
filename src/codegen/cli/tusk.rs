use proc_macro2::TokenStream;
use quote::quote;

use crate::Tusk;

impl Tusk {
    pub fn build_subcommand(&self, path_prefix_ident: &syn::Ident, path_sep: &str) -> TokenStream {
        let tusk_name = &self.name;

        let command_name_code = quote! {
            {
                let mut parts = #path_prefix_ident.clone();
                parts.push(#tusk_name.to_string());
                parts.join(#path_sep)
            }
        };

        let mut arg_statements = Vec::new();
        for (_, arg) in &self.arguments {
            arg_statements.push(arg.build_arg());
        }

        quote! {
            {
                let cmd_name = #command_name_code;
                let mut cmd = clap::Command::new(cmd_name);
                #(
                    cmd = cmd.arg(#arg_statements);
                )*
                cmd
            }
        }
    }
}

