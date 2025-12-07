use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{Argument, LinkNode, Tusk, TusksNode};

impl TusksNode {
    pub fn build_handle_matches(&self, path_sep: &str) -> TokenStream {
        // Sammle alle Match-Arms für alle Tusks im Baum
        let match_arms: Vec<TokenStream> = self.iter_all_tusks()
            .map(|(node, tusk)| {
                tusk.build_match_arm(node, path_sep)
            })
            .collect();
        
        // Sammle alle Link-Aufrufe für den fallback
        let link_calls: Vec<TokenStream> = self.iter_all_links()
            .map(|(node, link)| {
                Self::build_link_call(node, link)
            })
            .collect();
        
        quote! {
            pub fn handle_matches(matches: &clap::ArgMatches, link_path: Vec<String>) -> bool {
                let (subcommand_name, sub_matches) = matches.subcommand().unwrap();
                let subcommand_prefix = link_path.join(#path_sep);
                
                // Wenn der Command nicht mit unserem Prefix startet, returnen
                if !subcommand_prefix.is_empty() && !subcommand_name.starts_with(&subcommand_prefix) {
                    return false;
                }
                
                // Prefix vom Command-Namen entfernen
                let actual_command = if !subcommand_prefix.is_empty() {
                    let prefix_len = subcommand_prefix.len() + #path_sep.len();
                    &subcommand_name[prefix_len..]
                } else {
                    subcommand_name
                };
                
                match actual_command {
                    #(#match_arms)*
                    _ => {
                        #(#link_calls)*
                        eprintln!("Unknown command: {}", subcommand_name);
                        return true;
                    }
                }
            }
        }
    }
    
    fn build_link_call(node: &TusksNode, link: &LinkNode) -> TokenStream {
        let link_name = &link.name;

        // Build the path to the linked module's handle_matches function
        let relative_path: Vec<_> = node.module_path.iter().skip(1)
            .chain(std::iter::once(link_name))
            .collect();

        // Build the function path
        let mut path_parts = TokenStream::new();
        path_parts.extend(quote! { super });

        for segment in &relative_path {
            let segment_ident = syn::Ident::new(segment, Span::call_site());
            path_parts.extend(quote! { :: #segment_ident });
        }

        path_parts.extend(quote! { :: __tusks_internal_module :: handle_matches });

        // Build the link_path segments to push: module_path[1..] + link_name
        let segments_to_push: Vec<&String> = node.module_path.iter().skip(1)
            .chain(std::iter::once(link_name))
            .collect();

        quote! {
            {
                let mut new_link_path = link_path.clone();
                #(new_link_path.push(#segments_to_push.to_string());)*
                if #path_parts(matches, new_link_path) {
                    return true;
                }
            }
        }
    }
}

impl Tusk {
    pub fn build_match_arm(&self, node: &TusksNode, path_sep: &str) -> TokenStream {
        let tusk_name = &self.name;
        
        // Build the full command name from node's module_path (skip first element) + tusk name
        let path_parts: Vec<String> = node.module_path.iter()
            .skip(1)
            .chain(std::iter::once(tusk_name))
            .cloned()
            .collect();
        let command_name = path_parts.join(path_sep);
        
        // Build the path to the mirror function
        let mut mirror_path = TokenStream::new();
        mirror_path.extend(quote! { mirror_module });
        
        for segment in node.module_path.iter().skip(1) {
            let segment_ident = syn::Ident::new(segment, Span::call_site());
            mirror_path.extend(quote! { :: #segment_ident });
        }
        
        let tusk_ident = syn::Ident::new(tusk_name, Span::call_site());
        mirror_path.extend(quote! { :: #tusk_ident });
        
        // Extract arguments
        let arg_extractions: Vec<TokenStream> = self.arguments.iter()
            .enumerate()
            .map(|(i, (_, arg))| {
                let var_name = syn::Ident::new(&format!("arg_{}", i), Span::call_site());
                let extraction = arg.extract_from_matches();
                quote! {
                    let #var_name = #extraction;
                }
            })
            .collect();
        
        // Build the argument list for the function call
        let arg_vars: Vec<TokenStream> = (0..self.arguments.len())
            .map(|i| {
                let var_name = syn::Ident::new(&format!("arg_{}", i), Span::call_site());
                quote! { #var_name }
            })
            .collect();
        
        quote! {
            #command_name => {
                #(#arg_extractions)*
                #mirror_path(#(#arg_vars),*);
                return true;
            }
        }
    }
}

impl Argument {
    pub fn extract_from_matches(&self) -> proc_macro2::TokenStream {
        let arg_name = &self.name;

        if self.flag {
            // Flags are booleans
            quote! {
                sub_matches.get_flag(#arg_name)
            }
        } else if let Some(default) = &self.default {
            // Argument has a default value
            quote! {
                sub_matches
                    .get_one::<String>(#arg_name)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| #default.to_string())
            }
        } else if self.optional {
            // Optional arguments
            quote! {
                sub_matches.get_one::<String>(#arg_name).map(|s| s.clone())
            }
        } else {
            // Required arguments
            quote! {
                sub_matches.get_one::<String>(#arg_name).unwrap().clone()
            }
        }
    }
}
