use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::Tusk;

impl Tusk {
    pub fn create_mirror(&self, path: &[String]) -> TokenStream {
        let fn_name = syn::Ident::new(&self.name, Span::call_site());
        
        // Create function parameters - all as String or Option<String>
        let params: Vec<TokenStream> = self.arguments.iter().map(|(_, arg)| {
            arg.create_mirror_param()
        }).collect();
        
        // Build the path to the original function
        // Count supers: one for each element in path + 2 for mirror_module and __tusks_internal_module
        let super_count = path.len() + 2;
        
        // Build path as a single TokenStream
        let mut full_path = TokenStream::new();
        
        // Add all the supers
        for i in 0..super_count {
            if i > 0 {
                full_path.extend(quote! { :: });
            }
            full_path.extend(quote! { super });
        }
        
        // Add the module path
        for segment in path {
            full_path.extend(quote! { :: });
            let segment_ident = syn::Ident::new(segment, Span::call_site());
            full_path.extend(quote! { #segment_ident });
        }
        
        // Add the function name
        full_path.extend(quote! { :: #fn_name });
        
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
