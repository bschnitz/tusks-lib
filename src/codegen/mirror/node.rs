use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::TusksNode;

impl TusksNode {
    pub fn create_mirror(&self, path: &[String]) -> TokenStream{
        let mut items = Vec::new();

        // Create mirror functions for all tusks in this node
        for tusk in &self.tusks {
            items.push(tusk.create_mirror(path));
        }

        // Create mirror submodules for all child nodes
        for child in &self.childs {
            let child_name = syn::Ident::new(&child.get_module_name(), Span::call_site());
            let mut child_path = path.to_vec();
            child_path.push(child.get_module_name().clone());
            let child_mirror = child.create_mirror(&child_path);

            items.push(quote! {
                pub mod #child_name {
                    #child_mirror
                }
            });
        }

        quote! {
            #(#items)*
        }
    }
}
