use syn::ItemFn;

use crate::models::Tusk;

impl Tusk {
    pub fn from_fn(item_fn: ItemFn) -> syn::Result<Option<Self>> {
        // Only consider pub functions
        if !matches!(item_fn.vis, syn::Visibility::Public(_)) {
            return Ok(None);
        }
        
        // Validate return type is either nothing or an integer type
        Self::validate_return_type(&item_fn.sig.output)?;
        
        Ok(Some(Tusk {
            func: item_fn,
        }))
    }
    
    /// Validate that the return type is either nothing or an integer type
    fn validate_return_type(output: &syn::ReturnType) -> syn::Result<()> {
        match output {
            syn::ReturnType::Default => {
                // No return type - that's fine
                Ok(())
            }
            syn::ReturnType::Type(_, ty) => {
                // Check if it's an integer type
                if !Self::is_integer_type(ty) {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "command function must return nothing or an integer type (e.g., i32, u32, etc.)"
                    ));
                }
                Ok(())
            }
        }
    }
    
    /// Check if a type is an integer type
    fn is_integer_type(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = &segment.ident;
                matches!(
                    ident.to_string().as_str(),
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                )
            } else {
                false
            }
        } else {
            false
        }
    }
}
