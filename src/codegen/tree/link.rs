use crate::models::LinkNode;
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
}
