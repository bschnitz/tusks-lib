use syn::ItemFn;
use crate::parsing::util::attr::AttributeCheck;

use crate::models::Tusk;

impl Tusk {
    pub fn from_fn(item_fn: ItemFn, default_exists: bool) -> syn::Result<Option<Self>> {
        // Only consider pub functions
        if !matches!(item_fn.vis, syn::Visibility::Public(_)) || item_fn.has_attr("skip") {
            return Ok(None);
        }

        // Validate return type is either nothing, i32, or Option<i32>
        Self::validate_return_type(&item_fn.sig.output)?;

        let is_default = item_fn.has_attr("default");

        if is_default {
            Self::validate_default_function(&item_fn, default_exists)?;
        }

        Ok(Some(Tusk {
            func: item_fn,
            is_default
        }))
    }

    fn validate_default_function(item_fn: &ItemFn, default_exists: bool) -> syn::Result<()> {
        // Check for duplicate default attribute
        if default_exists {
            if let Some(attr) = item_fn.attrs.iter().find(|a| a.path().is_ident("default")) {
                return Err(syn::Error::new_spanned(
                    attr,
                    "only one function can be marked with #[default]"
                ));
            }
        }

        // Validate arguments: either no args or exactly one Parameters arg
        match item_fn.sig.inputs.len() {
            0 => Ok(()),
            1 => {
                let arg = &item_fn.sig.inputs[0];
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let syn::Type::Path(type_path) = &*pat_type.ty {
                        if type_path.path.is_ident("Parameters") {
                            return Ok(());
                        }
                    }
                }
                Err(syn::Error::new_spanned(
                    arg,
                    "default function must have either no arguments \
                        or exactly one argument of the Parameters type"
                ))
            }
            _ => Err(syn::Error::new_spanned(
                &item_fn.sig.inputs,
                "default function must have either no arguments \
                    or exactly one argument of the Parameters type"
            ))
        }
    }
    
    /// Validate that the return type is either nothing, u8, or Option<u8>
    fn validate_return_type(output: &syn::ReturnType) -> syn::Result<()> {
        match output {
            syn::ReturnType::Default => {
                // No return type - that's fine
                Ok(())
            }
            syn::ReturnType::Type(_, ty) => {
                // Check if it's u8 or Option<u8>
                if Self::is_u8_type(ty) || Self::is_option_u8_type(ty) {
                    Ok(())
                } else {
                    Err(syn::Error::new_spanned(
                        ty,
                        "command function must return (), u8, or Option<u8>"
                    ))
                }
            }
        }
    }

    /// Check if a type is u8
    pub fn is_u8_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "u8"
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if a type is Option<u8>
    pub fn is_option_u8_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(first_arg) = args.args.first() {
                            if let syn::GenericArgument::Type(inner_ty) = first_arg {
                                return Self::is_u8_type(inner_ty);
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
