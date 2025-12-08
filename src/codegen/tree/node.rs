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
        
        // create TusksNode from link by calling get_tusks_tree recursively
        let links_resolution = self.links.iter().map(|link| {
            link.link_to_node_resolution_tokens(path)
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
