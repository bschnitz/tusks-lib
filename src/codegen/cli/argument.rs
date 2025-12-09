use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_str;
use crate::Argument;

impl Argument {
    pub fn build_arg(&self) -> TokenStream {
        let arg_name = &self.name;
        let value_name_upper = self.name.to_uppercase();

        // Base Arg
        let mut arg_config = quote! { clap::Arg::new(#arg_name) };

        // Flag (boolean)
        if self.flag {
            arg_config = quote! {
                #arg_config
                    .action(clap::ArgAction::SetTrue)
            };
        } else {
            let type_token: syn::Type = parse_str(&self.type_)
                .expect(&format!("Invalid Rust type: {}", self.type_));

            arg_config = quote! {
                #arg_config
                    .value_name(#value_name_upper)
                    .value_parser(clap::value_parser!(#type_token))
            };

            // Optional / default / required
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

            // Multiplicity / num_args
            if let Some(m) = &self.count {
                match (m.min, m.max) {
                    (Some(min), Some(max)) => {
                        let min_lit = proc_macro2::Literal::usize_unsuffixed(min);
                        let max_lit = proc_macro2::Literal::usize_unsuffixed(max);
                        arg_config = quote! {
                            #arg_config
                                .num_args(#min_lit..=#max_lit)
                        };
                    }
                    (None, Some(max)) => {
                        let max_lit = proc_macro2::Literal::usize_unsuffixed(max);
                        arg_config = quote! {
                            #arg_config
                                .num_args(..=#max_lit)
                        };
                    }
                    (Some(min), None) => {
                        let min_lit = proc_macro2::Literal::usize_unsuffixed(min);
                        arg_config = quote! {
                            #arg_config
                                .num_args(#min_lit..)
                        };
                    }
                    (None, None) => {
                        arg_config = quote! {
                            #arg_config
                                .num_args(..)
                        };
                    }
                }
            }

            // Enum possibilities
            if let Some(enum_values) = &self.arg_enum {
                let enum_tokens = enum_values.iter().map(|v| quote! { #v });
                arg_config = quote! {
                    #arg_config
                        .value_parser([#(#enum_tokens),*])
                };
            }

            // Validator
            if let Some(validator_fn) = &self.validator {
                arg_config = quote! {
                    #arg_config
                        .validator(#validator_fn)
                };
            }
        }

        // Positional vs Named
        if self.positional {
            arg_config = quote! { #arg_config .index(1) };
        } else {
            arg_config = quote! { #arg_config .long(#arg_name) };
        }

        // Short
        if let Some(c) = self.short {
            arg_config = quote! { #arg_config .short(#c) };
        }

        // Help
        if let Some(help) = &self.help {
            arg_config = quote! { #arg_config .help(#help) };
        }

        // Hidden
        if self.hidden {
            arg_config = quote! { #arg_config .hide(true) };
        }

        // Value hint
        if let Some(hint) = &self.value_hint {
            let hint_tokens = hint.to_tokens();
            arg_config = quote! { #arg_config .value_hint(#hint_tokens) };
        }

        arg_config
    }
}
