use crate::models::TusksNode;
use proc_macro2::TokenStream;
use quote::quote;

impl TusksNode {
    pub fn to_tokens(&self, path: &[String]) -> TokenStream {
        let module_path = &self.module_path;
        let is_link = self.is_link;
        
        let tusks_code = self.tusks.iter().map(|tusk| tusk.to_tokens());
        
        // normal childs
        let childs_code = self.childs.iter().map(|child| {
            let mut child_path = path.to_vec();
            child_path.push(child.get_module_name().clone());
            child.to_tokens(&child_path)
        });
        
        // link to child node resolution using the appropriate functions
        let links_resolution = self.links.iter().map(|link| {
            let path_segments = if path.is_empty() {
                format!("super::{}", link.name)
            } else {
                format!("super::{}::{}", path.join("::"), link.name)
            };
            
            let path_tokens: TokenStream = path_segments.parse().unwrap();

            let link_name = &link.name;
            
            quote! {
                {
                    let mut linked = #path_tokens::__tusks_internal_module::get_tusks_tree();
                    linked.is_link = true;
                    linked.link_name = Some(#link_name.to_string());
                    linked
                }
            }
        });
        
        let links_code = self.links.iter().map(|link| link.to_tokens());
        
        quote! {
            {
                let mut node = tusks::TusksNode {
                    module_path: vec![#(#module_path.to_string()),*],
                    tusks: vec![#(#tusks_code),*],
                    childs: vec![#(#childs_code),*],
                    links: vec![#(#links_code),*],
                    is_link: #is_link,
                    link_name: None
                };
                
                // resolve links => child nodes
                #(node.childs.push(#links_resolution);)*
                
                node
            }
        }
    }
}
