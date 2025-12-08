use crate::models::Argument;
use proc_macro2::TokenStream;
use quote::quote;

impl Argument {
    pub fn to_tokens(&self) -> TokenStream {
        let name = &self.name;
        let type_ = &self.type_;
        let default = match &self.default {
            Some(d) => quote! { Some(#d.to_string()) },
            None => quote! { None },
        };
        let optional = self.optional;
        let flag = self.flag;
        let value = match &self.value {
            Some(v) => quote! { Some(#v.to_string()) },
            None => quote! { None },
        };

        quote! {
            tusks::Argument {
                name: #name.to_string(),
                type_: #type_.to_string(),
                default: #default,
                optional: #optional,
                flag: #flag,
                value: #value,
            }
        }
    }
}
