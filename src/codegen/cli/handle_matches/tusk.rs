use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn;

use crate::codegen::util::misc::build_path_tokens;
use crate::{Tusk, TusksNode};
use crate::codegen::cli::handle_matches::context::BuildContext;

impl Tusk {
    pub fn build_match_arm(&self, node: &TusksNode, ctx: &BuildContext) -> TokenStream {
        let tusk_name = &self.name;

        let path_to_tusk: Vec<_> = std::iter::empty()
            .chain(node.relative_module_path())
            .chain([tusk_name.as_str()])
            .collect();

        let command_name = path_to_tusk.join(ctx.path_sep);

        let mirror_path = std::iter::empty()
            .chain(["mirror_module"])
            .chain(path_to_tusk);

        // build mirror_module::path::to::tusk
        let mirror_path_tokens = build_path_tokens(mirror_path);

        // Builds "let arg_{i} = matches.get_one::<String>(#arg_name) ..." statements
        let arg_extractions: Vec<TokenStream> = self.arguments.iter()
            .enumerate()
            .map(|(i, (_, arg))| {
                let var_name = syn::Ident::new(&format!("arg_{}", i), Span::call_site());
                let extraction = arg.extract_from_matches(ctx.matches_var);
                quote! { let #var_name = #extraction; }
            })
            .collect();

        // Builds call to the tusk function with the arguments from above "fct(arg_1, arg_2, ...)"
        let arg_vars: Vec<TokenStream> = (0..self.arguments.len())
            .map(|i| syn::Ident::new(&format!("arg_{}", i), Span::call_site()))
            .map(|v| quote! { #v })
            .collect();

        // match for the command name will execute the tusk callback with cli parameters
        quote! {
            #command_name => {
                #(#arg_extractions)*
                #mirror_path_tokens(#(#arg_vars),*);
                return true;
            }
        }
    }
}

