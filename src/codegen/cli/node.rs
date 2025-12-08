use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::TusksNode;

impl TusksNode {
    pub fn build_cli(&self, command_var: &str, path_prefix_var: &str, path_sep: &str) -> TokenStream {
        let command_ident = syn::Ident::new(command_var, Span::call_site());
        let path_prefix_ident = syn::Ident::new(path_prefix_var, Span::call_site());

        let mut statements = Vec::new();

        // 1. Add subcommands for all tusks in this node
        for tusk in &self.tusks {
            let tusk_code = tusk.build_subcommand(&path_prefix_ident, path_sep);
            statements.push(quote! {
                let subcommand = #tusk_code;
                #command_ident = #command_ident.subcommand(subcommand);
            });
        }

        // 2. Handle link nodes (now via LinkNode method)
        for link in &self.links {
            statements.push(link.build_link_call(self, &command_ident, &path_prefix_ident));
        }

        // 3. Recursively handle child nodes
        for child in &self.childs {
            let child_module = child.get_module_name();

            statements.push(quote! {
                let mut child_prefix = #path_prefix_ident.clone();
                child_prefix.push(#child_module.to_string());
            });

            let child_build = child.build_cli(command_var, "child_prefix", path_sep);
            statements.push(child_build);
        }

        quote! {
            #(#statements)*
        }
    }
}
