use syn::ItemFn;
use crate::parsing::util::attr::AttributeCheck;

use crate::models::Tusk;

impl Tusk {
    pub fn from_fn(item_fn: ItemFn) -> syn::Result<Option<Self>> {
        // Only consider pub functions
        if !matches!(item_fn.vis, syn::Visibility::Public(_)) || item_fn.has_attr("skip") {
            return Ok(None);
        }
        
        // Validate return type is either nothing, i32, or Option<i32>
        Self::validate_return_type(&item_fn.sig.output)?;
        
        Ok(Some(Tusk {
            func: item_fn,
        }))
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
