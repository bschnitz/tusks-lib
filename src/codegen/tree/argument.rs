use crate::models::{Argument, ValueHintWrapper};
use clap::ValueHint;
use proc_macro2::TokenStream;
use quote::quote;

impl ValueHintWrapper {
    pub fn to_tokens(&self) -> TokenStream {
        match &self.0 {
            ValueHint::AnyPath => quote! { clap::ValueHint::AnyPath },
            ValueHint::FilePath => quote! { clap::ValueHint::FilePath },
            ValueHint::DirPath => quote! { clap::ValueHint::DirPath },
            ValueHint::ExecutablePath => quote! { clap::ValueHint::ExecutablePath },
            ValueHint::CommandName => quote! { clap::ValueHint::CommandName },
            ValueHint::CommandString => quote! { clap::ValueHint::CommandString },
            ValueHint::Username => quote! { clap::ValueHint::Username },
            ValueHint::Hostname => quote! { clap::ValueHint::Hostname },
            ValueHint::Url => quote! { clap::ValueHint::Url },
            ValueHint::EmailAddress => quote! { clap::ValueHint::EmailAddress },
            ValueHint::Other => quote! { clap::ValueHint::Other },
            _ => quote! { clap::ValueHint::Other },
        }
    }
}

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
        let positional = self.positional;

        let count = match &self.count {
            Some(c) => {
                let min = c.min.map(|v| {
                    let lit = proc_macro2::Literal::usize_unsuffixed(v);
                    quote! { Some(#lit) }
                }).unwrap_or(quote! { None });
                let max = c.max.map(|v| {
                    let lit = proc_macro2::Literal::usize_unsuffixed(v);
                    quote! { Some(#lit) }
                }).unwrap_or(quote! { None });
                quote! {
                    Some(tusks::ArgumentMultiplicity { min: #min, max: #max })
                }
            },
            None => quote! { None },
        };

        let short = match self.short {
            Some(c) => quote! { Some(#c) },
            None => quote! { None },
        };

        let help = match &self.help {
            Some(h) => quote! { Some(#h.to_string()) },
            None => quote! { None },
        };

        let hidden = self.hidden;

        let value_hint_tokens: TokenStream = match &self.value_hint {
            Some(hint) => hint.to_tokens(), // ruft die Methode auf
            None => quote! { None },
        };

        let arg_enum = match &self.arg_enum {
            Some(vals) => {
                let vals_iter = vals.iter().map(|v| quote! { #v.to_string() });
                quote! { Some(vec![#(#vals_iter),*]) }
            },
            None => quote! { None },
        };

        let validator = match &self.validator {
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
                positional: #positional,
                count: #count,
                short: #short,
                help: #help,
                hidden: #hidden,
                value_hint: #value_hint_tokens,
                arg_enum: #arg_enum,
                validator: #validator,
                arg: None
            }
        }
    }
}
