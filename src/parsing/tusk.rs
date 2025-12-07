use crate::models::{Argument, Tusk};
use crate::parsing::types::DefaultArguments;
use indexmap::IndexMap;
use syn::{Error, ItemFn};

impl Tusk {
    pub fn from_func(func: &ItemFn) -> Result<Self, Error> {
        let name = func.sig.ident.to_string();
        let mut arguments = IndexMap::new();

        // Extract defaults with span information
        let defaults = DefaultArguments::from_func(func)?;

        // Convert to the format expected by Argument::from_fn_arg
        let defaults_map = defaults.to_value_map();

        // Parse all function arguments
        for input in &func.sig.inputs {
            if let Some(argument) = Argument::from_fn_arg(input, &defaults_map)? {
                arguments.insert(argument.name.clone(), argument);
            }
        }

        defaults.validate_against_arguments(&arguments)?;
        defaults.validate_no_defaults_for_optional(&arguments)?;

        Ok(Tusk { name, arguments })
    }
}
