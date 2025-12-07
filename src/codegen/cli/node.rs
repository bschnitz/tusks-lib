use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{Argument, Tusk, TusksNode};

impl TusksNode {
    pub fn build_cli(&self, command_var: &str, path_prefix_var: &str, path_sep_var: &str) -> TokenStream {
        let command_ident = syn::Ident::new(command_var, Span::call_site());
        let path_prefix_ident = syn::Ident::new(path_prefix_var, Span::call_site());
        let path_sep_ident = syn::Ident::new(path_sep_var, Span::call_site());
        
        let mut statements = Vec::new();
        
        // Add subcommands for all tusks in this node
        for tusk in &self.tusks {
            let tusk_code = tusk.build_subcommand(&path_prefix_ident, &path_sep_ident);
            statements.push(quote! {
                let subcommand = #tusk_code;
                #command_ident = #command_ident.subcommand(subcommand);
            });
        }
        
        // Handle link nodes - call their build_cli with extended path_prefix
        for link in &self.links {
            let link_name = &link.name;
            
            // Build the path to the linked module's build_cli function
            // self.module_path contains the current module path (e.g. ["tasks", "docker"])
            // Skip the first element (root module name) and add the link name
            let relative_path: Vec<_> = self.module_path.iter().skip(1)
                .chain(std::iter::once(link_name))
                .collect();
            
            // Build the super:: prefix and module path
            let mut path_parts = TokenStream::new();
            
            // Start with super:: to get out of __tusks_internal_module
            path_parts.extend(quote! { super });
            
            // Add each module segment
            for segment in relative_path {
                let segment_ident = syn::Ident::new(segment, Span::call_site());
                path_parts.extend(quote! { :: #segment_ident });
            }
            
            // Add the internal module and build_cli function
            path_parts.extend(quote! { :: __tusks_internal_module :: build_cli });
            
            statements.push(quote! {
                let mut link_prefix = #path_prefix_ident.clone();
                if !link_prefix.is_empty() {
                    link_prefix.push(#path_sep_ident.clone());
                }
                link_prefix.push(#link_name.to_string());
                
                #command_ident = #path_parts(#command_ident, link_prefix, #path_sep_ident.clone());
            });
        }
        
        // Recursively handle child nodes
        for child in &self.childs {
            let child_module = child.get_module_name();
            
            // Build new path_prefix for child
            statements.push(quote! {
                let mut child_prefix = #path_prefix_ident.clone();
                if !child_prefix.is_empty() {
                    child_prefix.push(#path_sep_ident.clone());
                }
                child_prefix.push(#child_module.to_string());
            });
            
            let child_build = child.build_cli(command_var, "child_prefix", path_sep_var);
            statements.push(child_build);
        }
        
        quote! {
            #(#statements)*
        }
    }
}

impl Tusk {
    pub fn build_subcommand(&self, path_prefix_ident: &syn::Ident, path_sep_ident: &syn::Ident) -> TokenStream {
        let tusk_name = &self.name;
        
        // Build the full command name at runtime
        let command_name_code = quote! {
            {
                let mut parts = #path_prefix_ident.clone();
                if !parts.is_empty() {
                    parts.push(#path_sep_ident.clone());
                }
                parts.push(#tusk_name.to_string());
                parts.join("")
            }
        };
        
        // Build arguments
        let mut arg_statements = Vec::new();
        for (_, arg) in &self.arguments {
            arg_statements.push(arg.build_arg());
        }
        
        quote! {
            {
                let cmd_name = #command_name_code;
                let mut cmd = clap::Command::new(cmd_name);
                #(
                    cmd = cmd.arg(#arg_statements);
                )*
                cmd
            }
        }
    }
}

impl Argument {
    pub fn build_arg(&self) -> TokenStream {
        let arg_name = &self.name;
        
        let mut arg_config = quote! {
            clap::Arg::new(#arg_name)
        };
        
        // Handle flags
        if self.flag {
            arg_config = quote! {
                #arg_config.action(clap::ArgAction::SetTrue)
            };
        } else {
            // Regular arguments need a value - use the arg_name in uppercase as value_name
            let value_name_upper = self.name.to_uppercase();
            arg_config = quote! {
                #arg_config.value_name(#value_name_upper)
            };
        }
        
        // Handle optional arguments
        if !self.optional && !self.flag {
            arg_config = quote! {
                #arg_config.required(true)
            };
        } else if self.optional {
            arg_config = quote! {
                #arg_config.required(false)
            };
        }
        
        // Handle default values
        if let Some(default_val) = &self.default {
            arg_config = quote! {
                #arg_config.default_value(#default_val)
            };
        }
        
        // Add long flag format
        arg_config = quote! {
            #arg_config.long(#arg_name)
        };
        
        arg_config
    }
}
