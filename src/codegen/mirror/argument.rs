use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::Argument;

impl Argument {
    pub fn create_mirror_param(&self) -> TokenStream {
        let param_name = syn::Ident::new(&self.name, Span::call_site());
        
        if self.flag {
            // Flags are always bool
            quote! {
                #param_name: bool
            }
        } else if self.optional {
            quote! {
                #param_name: impl Into<Option<String>>
            }
        } else {
            quote! {
                #param_name: String
            }
        }
    }
    
    pub fn create_conversion(&self) -> TokenStream {
        let param_name = syn::Ident::new(&self.name, Span::call_site());
        
        if self.flag {
            // Flags are passed directly as bool
            quote! {
                #param_name
            }
        } else if self.optional {
            quote! {
                #param_name.into().map(|v: String| v.parse().unwrap())
            }
        } else {
            quote! {
                #param_name.parse().unwrap()
            }
        }
    }
}
