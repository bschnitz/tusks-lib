use crate::models::Argument;
use quote::quote;
use std::collections::HashMap;
use syn::{Error, FnArg};

impl Argument {
    /// Create an Argument from a function argument (FnArg).
    /// Returns Ok(None) for receiver arguments (which are reported as errors).
    /// Returns Err for invalid patterns or other errors.
    pub fn from_fn_arg(
        arg: &FnArg,
        defaults: &HashMap<String, String>,
    ) -> Result<Option<Self>, Error> {
        // Check for receiver (self) arguments
        if let FnArg::Receiver(receiver) = arg {
            return Err(Error::new_spanned(
                receiver,
                "Self receivers (&self, &mut self, self) are not supported in tusk functions",
            ));
        }

        // Only process typed arguments
        let pat_type = match arg {
            FnArg::Typed(pat_type) => pat_type,
            FnArg::Receiver(_) => unreachable!(), // Already handled above
        };

        // Extract argument name (only simple identifiers)
        let arg_name = match &*pat_type.pat {
            syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            _ => {
                return Err(Error::new_spanned(
                    &pat_type.pat,
                    "Complex patterns are not supported. Use simple argument names like 'name: String'",
                ));
            }
        };

        // Analyze type structure
        let (optional, inner_type) = Self::extract_option_inner(&pat_type.ty);

        // Use inner type if optional, otherwise the full type
        let type_display = quote!(#inner_type).to_string().replace(" ", "");

        // Look up default value for this argument
        let default = defaults.get(&arg_name).cloned();

        Ok(Some(Argument {
            name: arg_name,
            type_: type_display,
            default,
            optional,
            value: None,
        }))
    }

    /// Extract the inner type from Option<T>, returns (is_option, inner_type)
    /// If it's not Option<T>, ty will be returned as it is
    fn extract_option_inner(ty: &syn::Type) -> (bool, &syn::Type) {
        if let syn::Type::Path(type_path) = ty {
            // Check if this is a simple path (no self, no leading ::)
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let segment = &type_path.path.segments[0];

                // Check if the type name is "Option"
                if segment.ident == "Option" || 
                (type_path.path.segments.len() >= 2 && 
                type_path.path.segments.last().unwrap().ident == "Option") {
                    // Check for angle bracketed generic arguments
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        // Get the first generic argument
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return (true, inner_ty);
                        }
                    }
                }
            }
        }

        // Not an Option, return the original type
        (false, ty)
    }
}
