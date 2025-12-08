use crate::{codegen::util::misc::build_path_tokens, models::LinkNode};
use proc_macro2::TokenStream;
use quote::quote;

impl LinkNode {
    pub fn to_tokens(&self) -> TokenStream {
        let name = &self.name;

        quote! {
            tusks::LinkNode {
                name: #name.to_string(),
            }
        }
    }

    /// creates the call to get_tusks_tree for the link, such that the tree can be inserted as real
    /// TusksNode at the appropriate place
    pub fn link_to_node_resolution_tokens(&self, path: &[String]) -> TokenStream {
        let path_tokens = build_path_tokens(
            std::iter::once("super")
                .chain(path.iter().map(|s| s.as_str()))
                .chain(std::iter::once(self.name.as_str()))
                .chain(std::iter::once("__tusks_internal_module"))
        );
        
        let link_name = &self.name;
        
        quote! {
            {
                let mut linked = #path_tokens::get_tusks_tree();
                linked.is_link = true;
                linked.link_name = Some(#link_name.to_string());
                linked
            }
        }
    }
}
