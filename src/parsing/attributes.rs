use syn::{Error, ItemFn};
use syn::spanned::Spanned;
use indexmap::IndexMap;
use crate::{Argument, parsing::util::types::{
    ArgumentKeyValueAttributes, ArgumentListAttributes, FunctionAttributes
}};

/// Represents the parsed attributes for a Tusk function.
///
/// This struct wraps `FunctionAttributes` and provides Tusk-specific parsing
/// and validation logic for function attributes like `#[defaults(...)]` and
/// `#[positional(...)]`.
///
/// # Example
///
/// ```rust
/// #[defaults(user_id = 42, verbose = false)]
/// #[positional(required, user_id)]
/// fn run(required: String, user_id: u32, verbose: bool, name: String) {}
///
/// let tusk_attrs = TuskFunctionAttributes::from_func(&func)?;
/// tusk_attrs.validate_no_defaults_for_optional(&arguments)?;
/// ```
#[derive(Debug, Clone)]
pub struct TuskFunctionAttributes {
    attributes: FunctionAttributes,
}

impl TuskFunctionAttributes {
    /// Parses `#[defaults(...)]` and `#[positional(...)]` attributes from a function.
    ///
    /// This function extracts two specific attribute types:
    /// - `#[defaults(arg1 = "value", ...)]` - key-value pairs for default values
    /// - `#[positional(arg1, arg3, ...)]` - list of positional arguments
    ///
    /// Both attribute types are validated for correct syntax and checked for duplicates.
    ///
    /// # Errors
    ///
    /// Returns a `syn::Error` if:
    /// - Attribute syntax is invalid
    /// - Duplicate keys/identifiers are found within the same attribute
    /// - Token parsing fails
    pub fn from_func(func: &ItemFn) -> Result<Self, Error> {
        let mut function_attrs = FunctionAttributes::new();

        // Parse #[defaults(key = value, ...)]
        let defaults = ArgumentKeyValueAttributes::from_func(func, "defaults")?;
        if defaults.argument_names().next().is_some() {
            function_attrs.insert_key_value_set("defaults".to_string(), defaults);
        }

        // Parse #[positional(arg1, arg2, ...)]
        let positional = ArgumentListAttributes::from_func(func, "positional")?;
        if !positional.is_empty() {
            function_attrs.insert_list_set("positional".to_string(), positional);
        }

        Ok(Self {
            attributes: function_attrs,
        })
    }

    /// Validates that no default values are specified for optional arguments.
    ///
    /// This validation ensures that arguments marked as `optional` in the function
    /// signature do not have default values specified via the `#[defaults(...)]` attribute.
    /// Optional arguments already have a default (None), so specifying an additional
    /// default value would be ambiguous or conflicting.
    ///
    /// # Arguments
    ///
    /// * `arguments` - A map of argument names to their `Argument` definitions
    ///
    /// # Errors
    ///
    /// Returns a `syn::Error` if any argument that:
    /// - Has a default value specified in `#[defaults(...)]`
    /// - Is marked as `optional = true` in the arguments map
    ///
    /// The error message includes the argument name and references the span where
    /// the default value was specified.
    ///
    /// # Example
    ///
    /// ```rust
    /// // This would produce an error:
    /// #[defaults(user_id = 42)]  // Error: user_id is optional
    /// fn run(user_id: Option<u32>) {}
    /// ```
    pub fn validate_no_defaults_for_optional(
        &self,
        arguments: &IndexMap<String, Argument>,
    ) -> Result<(), Error> {
        if let Some(defaults) = self.attributes.get_key_value_set("defaults") {
            for (arg_name, attr) in defaults.iter() {
                if let Some(argument) = arguments.get(arg_name) {
                    if argument.optional {
                        return Err(Error::new(
                            attr.key_span,
                            format!(
                                "Cannot specify default value for optional argument '{}'. \
                                Optional arguments already have an implicit default value (None)",
                                arg_name
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Provides access to the underlying `FunctionAttributes`.
    pub fn attributes(&self) -> &FunctionAttributes {
        &self.attributes
    }

    /// Provides mutable access to the underlying `FunctionAttributes`.
    pub fn attributes_mut(&mut self) -> &mut FunctionAttributes {
        &mut self.attributes
    }

    pub fn add_attribute_info(
        &self,
        arguments: &mut IndexMap<String, Argument>,
    ) -> Result<(), Error> {
        let attrs = &self.attributes;

        if let Some(defaults) = attrs.get_key_value_set("defaults") {
            for (arg_name, kv) in defaults.iter() {
                let arg = arguments.get_mut(arg_name).ok_or_else(|| {
                    Error::new(kv.key_span, format!("#[defaults] references unknown argument `{}`", arg_name))
                })?;

                // Optional check: don't allow defaults for Option<T>
                if arg.optional {
                    let mut err1 = Error::new(
                        kv.key_span,
                        format!("Cannot set default for optional argument `{}`", arg_name)
                    );

                    // Only add second error if FnArg exists
                    if let Some(fnarg) = arg.arg.as_ref() {
                        let err2 = Error::new(
                            fnarg.0.span(),
                            format!("The argument was specified as Option here `{}`", arg_name)
                        );
                        err1.combine(err2);
                    }

                    return Err(err1);
                }

                arg.default = Some(kv.value.clone());
            }
        }

        if let Some(positional) = attrs.get_list_set("positional") {
            for item in positional.iter() {
                let arg = arguments.get_mut(&item.argument_name).ok_or_else(|| {
                    Error::new(item.span, format!("#[positional] references unknown argument `{}`", item.argument_name))
                })?;

                arg.positional = true;
            }
        }

        Ok(())
    }
}
