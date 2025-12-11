use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::codegen::util::enum_util::{convert_submodule_to_enum_variant, convert_external_module_to_enum_variant};

use crate::{TusksModule, models::Tusk};

impl TusksModule {
    /// Generate the handle_matches function
    pub fn build_handle_matches(&self, is_tusks_root: bool) -> TokenStream {
        let signature = if is_tusks_root {
            quote! {
                pub fn handle_matches(cli: &cli::Cli)
            }
        } else {
            quote! {
                pub fn handle_matches(cli: &cli::Cli, super_parameters: &super::parent_::Parameters)
            }
        };
        
        let params_init = self.build_parameters_initialization();
        let match_arms = self.build_match_arms_recursive(&[]);
        
        quote! {
            #signature {
                #params_init
                
                let commands = &cli.sub;
                match commands {
                    #(#match_arms)*
                    
                    None => {
                        println!("No function defined for this command!");
                    }
                }
            }
        }
    }
    
    fn build_parameters_initialization(&self) -> TokenStream {
        if let Some(ref params) = self.parameters {
            let mut field_inits = Vec::new();
            
            for field in &params.pstruct.fields {
                if let Some(field_name) = &field.ident {
                    if field_name == "super_" {
                        field_inits.push(quote! { super_: super_parameters, });
                    } else {
                        field_inits.push(quote! { #field_name: &cli.#field_name, });
                    }
                }
            }
            
            quote! {
                let parameters = super::Parameters {
                    #(#field_inits)*
                };
            }
        } else {
            quote! {}
        }
    }
    
    /// Build match arms recursively with path tracking
    fn build_match_arms_recursive(&self, path: &[&str]) -> Vec<TokenStream> {
        let mut arms = Vec::new();

        // Build cli path
        let cli_path = if path.is_empty() {
            quote! { cli }
        } else {
            let path_idents: Vec<_> = path.iter()
                .map(|p| syn::Ident::new(p, Span::call_site()))
                .collect();
            quote! { cli::#(#path_idents)::* }
        };

        // Arms for tusks
        for tusk in &self.tusks {
            arms.push(self.build_function_match_arm(tusk, &cli_path, path));
        }

        // Arms for submodules
        for submodule in &self.submodules {
            arms.push(self.build_submodule_arm(submodule, &cli_path, path));
        }

        // Arm for external commands (at ANY level, not just root!)
        if !self.external_modules.is_empty() {
            arms.push(self.build_external_arm(&cli_path, path));
        }

        arms
    }
    
    fn build_submodule_arm(&self, submodule: &TusksModule, cli_path: &TokenStream, path: &[&str]) -> TokenStream {
        let submod_name = &submodule.name;
        let submod_name_str = submod_name.to_string();
        let variant_ident = convert_submodule_to_enum_variant(submod_name);

        // Build pattern bindings for submodule parameters
        let mut pattern_bindings = Vec::new();
        let mut param_counter = 1;

        if let Some(ref params) = submodule.parameters {
            for field in &params.pstruct.fields {
                if let Some(field_name) = &field.ident {
                    if field_name != "super_" {
                        let binding_name = syn::Ident::new(&format!("p{}", param_counter), Span::call_site());
                        pattern_bindings.push((field_name.clone(), binding_name.clone()));
                        param_counter += 1;
                    }
                }
            }
        }

        // Build pattern for match arm
        let mut pattern_fields: Vec<_> = pattern_bindings.iter()
            .map(|(field_name, binding_name)| {
                quote! { #field_name: #binding_name }
            })
            .collect();

        // Check if submodule has any commands
        let has_commands = !submodule.tusks.is_empty() || 
        !submodule.submodules.is_empty() || 
        !submodule.external_modules.is_empty();

        // Add sub field binding only if there are commands
        if has_commands {
            pattern_fields.push(quote! { sub });
        }

        // Build submodule parameter initialization
        let params_init = if let Some(ref params) = submodule.parameters {
            let mut field_inits = Vec::new();

            for field in &params.pstruct.fields {
                if let Some(field_name) = &field.ident {
                    if field_name == "super_" {
                        field_inits.push(quote! { super_: super_parameters, });
                    } else {
                        // Find the binding for this field
                        if let Some((_, binding_name)) = pattern_bindings.iter()
                            .find(|(fname, _)| fname == field_name) {
                            field_inits.push(quote! { #field_name: #binding_name, });
                        }
                    }
                }
            }

            // Build parameters path
            let params_path = if path.is_empty() {
                quote! { super::#submod_name::Parameters }
            } else {
                let path_idents: Vec<_> = path.iter()
                    .map(|p| syn::Ident::new(p, Span::call_site()))
                    .collect();
                quote! { super::#(#path_idents)::*::#submod_name::Parameters }
            };

            quote! {
                let super_parameters = &parameters;
                let parameters = #params_path {
                    #(#field_inits)*
                };
            }
        } else {
            quote! {}
        };

        // Build new path for nested recursion
        let mut new_path = path.to_vec();
        new_path.push(&submod_name_str);

        // Get nested match arms (only if has commands)
        let nested_match = if has_commands {
            let nested_arms = submodule.build_match_arms_recursive(&new_path);

            quote! {
                match sub {
                    #(#nested_arms)*

                    None => {
                        println!("No function defined for this command!");
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {
            Some(#cli_path::Commands::#variant_ident { #(#pattern_fields),* }) => {
                #params_init
                #nested_match
            }
        }
    }
    
    fn build_external_arm(&self, cli_path: &TokenStream, path: &[&str]) -> TokenStream {
        let mut external_arms = Vec::new();

        for ext_mod in &self.external_modules {
            let alias = &ext_mod.alias;
            let variant_ident = convert_external_module_to_enum_variant(alias);

            // Build the correct path to the external module's handle_matches
            // If we're at root: super::#alias
            // If we're nested: super::#path[0]::#path[1]::...#alias
            let external_path = if path.is_empty() {
                // At root level
                quote! { super::#alias }
            } else {
                // Nested level - need to build full path from root
                let path_idents: Vec<_> = path.iter()
                    .map(|p| syn::Ident::new(p, Span::call_site()))
                    .collect();
                quote! { super::#(#path_idents)::*::#alias }
            };

            external_arms.push(quote! {
                #cli_path::ExternalCommands::#variant_ident(cli) => {
                    #external_path::__internal_tusks_module::handle_matches(cli, &parameters);
                }
            });
        }

        quote! {
            Some(#cli_path::Commands::TuskExternalCommands(commands)) => {
                match commands {
                    #(#external_arms)*
                }
            }
        }
    }
    
    pub fn tusk_has_parameters_arg(&self, tusk: &Tusk) -> bool {
        if let Some(syn::FnArg::Typed(first_param)) = tusk.func.sig.inputs.first() {
            if let Some(ref params) = self.parameters {
                return Self::is_parameters_type(&first_param.ty, &params.pstruct.ident);
            }
        }
        false
    }
}
