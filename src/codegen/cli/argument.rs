use proc_macro2::TokenStream;
use quote::quote;

use crate::Argument;

impl Argument {
    pub fn build_arg(&self) -> TokenStream {
        let arg_name = &self.name;
        let value_name_upper = self.name.to_uppercase();

        let mut arg_config = quote! {
            clap::Arg::new(#arg_name)
        };

        if self.flag {
            arg_config = quote! {
                #arg_config
                    .action(clap::ArgAction::SetTrue)
            };
        } else {
            arg_config = quote! {
                #arg_config
                    .value_name(#value_name_upper)
            };

            if let Some(default_val) = &self.default {
                arg_config = quote! {
                    #arg_config
                        .default_value(#default_val)
                };
            } else if self.optional {
                arg_config = quote! {
                    #arg_config
                        .required(false)
                };
            } else {
                arg_config = quote! {
                    #arg_config
                        .required(true)
                };
            }
        }

        arg_config = quote! {
            #arg_config
                .long(#arg_name)
        };

        arg_config
    }
}
