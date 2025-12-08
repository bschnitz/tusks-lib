use crate::models::{Argument, Tusk};
use crate::parsing::attributes::TuskFunctionAttributes;
use indexmap::IndexMap;
use syn::{Error, ItemFn};

impl Tusk {
    pub fn from_func(func: &ItemFn) -> Result<Self, Error> {
        let name = func.sig.ident.to_string();
        let mut arguments = IndexMap::new();

        let attributes = TuskFunctionAttributes::from_func(func)?;

        // Parse all function arguments
        for input in &func.sig.inputs {
            if let Some(argument) = Argument::from_fn_arg(input)? {
                arguments.insert(argument.name.clone(), argument);
            }
        }

        attributes.add_attribute_info(&mut arguments)?;

        Ok(Tusk { name, arguments })
    }
}
