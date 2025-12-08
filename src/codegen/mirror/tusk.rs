use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::iter;
use crate::{Tusk, codegen::util::misc::build_path_tokens};

impl Tusk {
    pub fn create_mirror(&self, path: &[String]) -> TokenStream {
        let fn_name = syn::Ident::new(&self.name, Span::call_site());
        
        // Create function parameters - all as String or Option<String>
        let params: Vec<TokenStream> = self.arguments.iter().map(|(_, arg)| {
            arg.create_mirror_param()
        }).collect();
        
        // Build the path to the original function
        // Count supers:
        // one for each element in path + 2 for mirror_module and __tusks_internal_module
        let super_count = path.len() + 2;
        
        // Build path as iterator and convert to TokenStream
        let full_path = build_path_tokens(
            iter::repeat("super")
                .take(super_count)
                .chain(path.iter().map(|s| s.as_str()))
                .chain(iter::once(self.name.as_str()))
        );
        
        // Create argument conversions for the function call
        let arg_conversions: Vec<TokenStream> = self.arguments.iter().map(|(_, arg)| {
            arg.create_conversion()
        }).collect();
        
        quote! {
            pub fn #fn_name(#(#params),*) {
                #full_path(#(#arg_conversions),*)
            }
        }
    }
}
