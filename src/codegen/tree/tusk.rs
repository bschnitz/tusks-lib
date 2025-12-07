use crate::models::Tusk;
use proc_macro2::TokenStream;
use quote::quote;

impl Tusk {
    pub fn to_tokens(&self) -> TokenStream {
        let name = &self.name;
        
        let args_code = self.arguments.iter().map(|(arg_name, arg)| {
            let arg_tokens = arg.to_tokens();
            quote! {
                map.insert(#arg_name.to_string(), #arg_tokens);
            }
        });
        
        quote! {
            tusks::Tusk {
                name: #name.to_string(),
                arguments: {
                    let mut map = std::collections::HashMap::new();
                    #(#args_code)*
                    map
                },
            }
        }
    }
}
