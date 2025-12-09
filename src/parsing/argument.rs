use crate::models::{Argument, ArgumentMultiplicity, NoDebug};
use quote::quote;
use syn::{Error, FnArg, GenericArgument, PathArguments, Type, TypePath};

impl Argument {
    /// Create an Argument from a function argument (FnArg).
    /// Returns Ok(None) for receiver arguments (which are reported as errors).
    /// Returns Err for invalid patterns or other errors.
    pub fn from_fn_arg(arg: &FnArg) -> Result<Option<Self>, Error> {
        let pat_type = match arg {
            FnArg::Receiver(receiver) => {
                return Err(Error::new_spanned(
                    receiver,
                    "Self receivers (&self, &mut self, self) are not supported in tusk functions",
                ));
            }
            FnArg::Typed(pat_type) => pat_type,
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

        let mut argument = Argument {
            name: arg_name,
            type_: "".to_string(),
            default: None,
            optional: false,
            flag: false,
            positional: false,
            count: None,
            short: None,
            help: None,
            hidden: false,
            value_hint: None,
            arg_enum: None,
            validator: None,
            arg: Some(NoDebug(arg.clone()))
        };

        argument.parse_type(&pat_type.ty)?;

        Ok(Some(argument))
    }

    /// Main function: Parses the type and fills the Argument structure
    pub fn parse_type(&mut self, ty: &syn::Type) -> syn::Result<()> {
        let mut current_type = ty;

        // Step 1: Unwrap Option<T>
        if let Some(inner) = extract_option_inner(current_type) {
            self.optional = true;
            current_type = inner;
        }

        // Step 2: Handle Vec/Repeat*
        if let Some((inner, multiplicity)) = extract_collection_inner(current_type)? {
            self.count = Some(multiplicity);
            current_type = inner;
        }

        // Step 3: Check final type
        if is_bool_type(current_type) {
            // bool is only allowed as a simple flag
            if self.optional || self.count.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "bool type can only be used for simple flags (not with Option or multiplicity)"
                ));
            }
            self.flag = true;
            self.type_ = "bool".to_string();
        } else {
            // Convert type to string
            self.type_ = quote!(#current_type).to_string().trim().to_string();
        }

        Ok(())
    }
}


/// Extracts the inner type from Option<T>
fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

/// Extracts the inner type and multiplicity from Vec/Repeat*
fn extract_collection_inner(ty: &Type) -> syn::Result<Option<(&Type, ArgumentMultiplicity)>> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            let type_name = segment.ident.to_string();

            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                let args_vec: Vec<_> = args.args.iter().collect();

                // Extract inner type T (always the first argument)
                let inner_type = if let Some(GenericArgument::Type(inner)) = args_vec.first() {
                    inner
                } else {
                    return Ok(None);
                };

                // Check for Vec, Repeat, RepeatMin, RepeatMax, RepeatMinMax
                let multiplicity = match type_name.as_str() {
                    "Vec" => ArgumentMultiplicity { min: None, max: None },
                    "Repeat" => {
                        // Repeat<T, N> - exactly N times
                        let n = if let Some(arg) = args_vec.get(1) {
                            extract_const_usize(arg)?
                        } else {
                            None
                        };
                        ArgumentMultiplicity { min: n, max: n }
                    },
                    "RepeatMin" => {
                        // RepeatMin<T, MIN>
                        let min = if let Some(arg) = args_vec.get(1) {
                            extract_const_usize(arg)?
                        } else {
                            None
                        };
                        ArgumentMultiplicity { min, max: None }
                    },
                    "RepeatMax" => {
                        // RepeatMax<T, MAX>
                        let max = if let Some(arg) = args_vec.get(1) {
                            extract_const_usize(arg)?
                        } else {
                            None
                        };
                        ArgumentMultiplicity { min: None, max }
                    },
                    "RepeatMinMax" => {
                        // RepeatMinMax<T, MIN, MAX>
                        let min = if let Some(arg) = args_vec.get(1) {
                            extract_const_usize(arg)?
                        } else {
                            None
                        };
                        let max = if let Some(arg) = args_vec.get(2) {
                            extract_const_usize(arg)?
                        } else {
                            None
                        };
                        ArgumentMultiplicity { min, max }
                    },
                    _ => return Ok(None),
                };

                return Ok(Some((inner_type, multiplicity)));
            }
        }
    }
    Ok(None)
}

/// Extracts a usize value from a const generic argument
fn extract_const_usize(arg: &GenericArgument) -> syn::Result<Option<usize>> {
    if let GenericArgument::Const(expr) = arg {
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit_int), .. }) = expr {
            return Ok(Some(lit_int.base10_parse::<usize>()?));
        }
    }
    Ok(None)
}

/// Checks if the type is `bool`
fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == 1 {
            if let Some(segment) = path.segments.first() {
                return segment.ident == "bool";
            }
        }
    }
    false
}
