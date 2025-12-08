use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::cli::handle_matches::context::BuildContext;
use crate::{LinkNode, TusksNode};

impl LinkNode {
    /// Build the call to the linked module's handle_matches
    pub fn build_link_handle_matches(&self, parent: &TusksNode, ctx: &BuildContext) -> TokenStream {
        let link_name = &self.name;

        let handle_matches_clb = self.build_internal_module_path(parent, "handle_matches");

        // Build link_path segments to push (module path + link_name)
        let segments_to_push = std::iter::empty()
            .chain(parent.relative_module_path())
            .chain([link_name.as_str()]);

        let link_path_var = ctx.link_path_var;
        let matches_var = ctx.matches_var;

        quote! {
            {
                let mut new_link_path = #link_path_var.clone();
                #(new_link_path.push(#segments_to_push.to_string());)*
                if #handle_matches_clb(#matches_var, new_link_path) {
                    return true;
                }
            }
        }
    }
}

