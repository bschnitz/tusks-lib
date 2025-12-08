use proc_macro2::TokenStream;
use quote::quote;

use crate::{LinkNode, TusksNode};

impl LinkNode {
    pub fn build_link_call(
        &self,
        parent: &TusksNode,
        command_ident: &syn::Ident,
        path_prefix_ident: &syn::Ident,
    ) -> TokenStream {
        // Use the unified utility function
        let module_path_ts = self.build_internal_module_path(parent, "build_cli");
        let link_name = &self.name;

        quote! {
            {
                let mut link_prefix = #path_prefix_ident.clone();
                link_prefix.push(#link_name.to_string());

                #command_ident = #module_path_ts(#command_ident, link_prefix);
            }
        }
    }
}
