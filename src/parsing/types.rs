use crate::models::{Argument};
use std::collections::HashMap;
use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Error, ItemFn, Attribute, spanned::Spanned};

#[derive(Debug, Clone)]  
pub struct DefaultArgument {  
    pub name: String,  
    pub value: String,  
    pub name_span: proc_macro2::Span,  
}  

#[derive(Debug, Clone, Default)]
pub struct DefaultArguments {
    defaults: HashMap<String, DefaultArgument>,
}

type TokenIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

impl DefaultArguments {
    fn new() -> Self {
        Self {
            defaults: HashMap::new(),
        }
    }

    /// Parse all #[defaults(...)] attributes from a function
    pub fn from_func(func: &ItemFn) -> Result<Self, Error> {
        let mut collection = Self::new();
        
        for attr in &func.attrs {
            if attr.path().is_ident("defaults") {
                collection.parse_attribute(attr)?;
            }
        }
        
        Ok(collection)
    }

    /// Parse a single #[defaults(...)] attribute and add to collection
    fn parse_attribute(&mut self, attr: &Attribute) -> Result<(), Error> {
        let meta_list = attr.meta.require_list().map_err(|_| {
            Error::new_spanned(
                attr,
                "Invalid defaults attribute format. Expected: #[defaults(key=\"value\", ...)]",
            )
        })?;

        self.parse_key_value_pairs(&meta_list.tokens, attr)
    }

    /// Parse key="value" pairs from token stream with strict comma-separated syntax
    /// 
    /// Validates the format: `key1="value1", key2="value2", ...`
    /// - Ensures '=' follows each key
    /// - Requires string literals for values
    /// - Enforces commas between pairs
    /// - Detects duplicate keys with helpful error messages
    fn parse_key_value_pairs(&mut self, tokens: &TokenStream, attr: &Attribute) -> Result<(), Error> {
        let mut iter = tokens.clone().into_iter().peekable();
        let mut last_span = attr.span();

        loop {
            self.digest_key_value(&mut iter, &mut last_span)?;

            if ! iter.peek().is_some() {
                break;
            }
            
            self.digest_comma(&mut iter, &mut last_span)?;
        }

        Ok(())
    }

    /// Consume and parse a key="value" pair from the iterator
    /// Expected format: identifier '=' string_literal
    fn digest_key_value(&mut self, iter: &mut TokenIter, last_span: &mut Span) -> Result<(), Error> {
        let (key, key_span) = self.digest_key(iter, last_span)?;
        self.digest_equal_sign(iter, &key, last_span)?;
        let value = self.digest_value(iter, &key, last_span)?;

        let default_arg = DefaultArgument {
            name: key,
            value,
            name_span: key_span,
        };

        self.insert_with_duplicate_check(default_arg)?;

        Ok(())
    }

    /// Consume and return an identifier (key name)
    fn digest_key(&self, iter: &mut TokenIter, last_span: &mut Span) -> Result<(String, Span), Error>
    {
        match iter.next() {
            Some(TokenTree::Ident(ident)) => {
                *last_span = ident.span();
                Ok((ident.to_string(), ident.span()))
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other, "Expected identifier (key name)"))
            }
            None => Err(Error::new(*last_span, "Expected identifier (key name), found end of tokens")),
        }
    }

    /// Consume and validate an '=' sign
    fn digest_equal_sign(&self, iter: &mut TokenIter, key: &str, last_span: &mut Span) -> Result<(), Error>
    {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                *last_span = punct.span();
                Ok(())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other, format!("Expected '=' after key '{}'", key)))
            }
            None => Err(Error::new(*last_span, format!("Expected '=' after key '{}'", key))),
        }
    }

    /// Consume and return a string literal value
    fn digest_value(&self, iter: &mut TokenIter, key: &str, last_span: &mut Span) -> Result<String, Error>
    {
        match iter.next() {
            Some(TokenTree::Literal(lit)) => {
                *last_span = lit.span();
                let value_str = lit.to_string();
                if value_str.starts_with('"') && value_str.ends_with('"') {
                    Ok(value_str.trim_matches('"').to_string())
                } else {
                    Err(Error::new_spanned(
                        lit,
                        format!("Default value for '{}' must be a quoted string", key),
                    ))
                }
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other, format!("Expected string value after '{}='", key)))
            }
            None => Err(Error::new(*last_span, format!("Expected string value after '{}='", key))),
        }
    }

    /// Consume and validate a comma from the iterator
    fn digest_comma(&mut self, iter: &mut TokenIter, last_span: &mut Span) -> Result<(), Error> {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                *last_span = punct.span();
                Ok(())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other, "Expected comma between key-value pairs"))
            }
            None => Err(Error::new(*last_span, "Expected comma, found end of tokens")),
        }
    }

    /// Insert default argument and check for duplicates
    fn insert_with_duplicate_check(&mut self, default_arg: DefaultArgument) -> Result<(), Error> {
        if let Some(existing) = self.defaults.get(&default_arg.name) {
            let mut err1 = Error::new(
                default_arg.name_span,
                format!("default value for argument '{}' specified more than once", default_arg.name)
            );
            let err2 = Error::new(
                existing.name_span,
                format!("first specification of default value for '{}'", default_arg.name)
            );
            err1.combine(err2);
            return Err(err1);
        }

        self.defaults.insert(default_arg.name.clone(), default_arg);
        Ok(())
    }

    pub fn to_value_map(&self) -> HashMap<String, String> {
        self.defaults
            .iter()
            .map(|(name, default_arg)| (name.clone(), default_arg.value.clone()))
            .collect()
    }

    /// Validate that all default values refer to existing arguments
    pub fn validate_against_arguments(
        &self,
        arguments: &IndexMap<String, Argument>,
    ) -> Result<(), Error> {
        for default in self.defaults.values() {
            if !arguments.contains_key(&default.name) {
                let available_args: Vec<_> = arguments.keys().cloned().collect();
                let available_str = if available_args.is_empty() {
                    "none".to_string()
                } else {
                    available_args.join(", ")
                };

                return Err(Error::new(
                    default.name_span,
                    format!(
                        "Default value for unknown argument '{}'. Available arguments: {}",
                        default.name, available_str
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Validate that optional arguments (Option<T>) don't have default values
    pub fn validate_no_defaults_for_optional(
        &self,
        arguments: &IndexMap<String, Argument>,
    ) -> Result<(), Error> {
        for default in self.defaults.values() {
            if let Some(arg) = arguments.get(&default.name) {
                if arg.optional {
                    return Err(Error::new(
                        default.name_span,
                        format!(
                            "Argument '{}' is optional (Option<T>) and cannot have a default value. Use either Option<T> OR a default value, not both.",
                            default.name
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}
