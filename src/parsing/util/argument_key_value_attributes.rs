use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Error, ItemFn, Attribute, spanned::Spanned};

use crate::parsing::util::types::ArgumentKeyValueAttributes;

type TokenIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

impl ArgumentKeyValueAttributes {
    /// Parses key-value-style attributes from a function where arguments are listed
    /// together with values, separated by commas.
    ///
    /// Each parsed key corresponds to the *name of a function argument*.
    /// After parsing, the macro validates that every attribute key matches
    /// an actual argument of the function. Unknown argument names result
    /// in a precise, span-based compiler error.
    ///
    /// Each key is stored together with:
    /// - its normalized string value
    /// - the span at which the key was defined (used for error reporting)
    ///
    /// The outer map `attributes` maps each key (`String`)
    /// to a full `ArgumentKeyValueAttribute` containing value and span.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[defaults(user_id = 42, verbose = false)]
    /// fn run(required: String, user_id: u32, verbose: bool) {}
    ///
    /// let attrs = ArgumentKeyValueAttributes::from_func(&func, "defaults")?;
    ///
    /// assert_eq!(attrs.get("user_id").unwrap().value, "42");
    /// assert_eq!(attrs.get("verbose").unwrap().value, "false");
    /// ```
    ///
    /// This struct is intentionally designed so macros can:
    /// - validate attribute keys *against actual function arguments*
    /// - detect duplicates
    /// - give precise compiler errors referencing the original source span.
    pub fn from_func(func: &ItemFn, attr_name: &str) -> Result<Self, Error> {
        let mut collection = Self::new();

        for attr in &func.attrs {
            if attr.path().is_ident(attr_name) {
                collection.parse_attribute(attr, attr_name)?;
            }
        }

        Ok(collection)
    }

    fn parse_attribute(&mut self, attr: &Attribute, attr_name: &str) -> Result<(), Error> {
        let meta_list = attr.meta.require_list().map_err(|_| {
            Error::new_spanned(
                attr,
                format!("Invalid {} attribute format. Expected: #[{}(key=value, ...)]",
                    attr_name, attr_name),
            )
        })?;

        self.parse_key_value_pairs(&meta_list.tokens, attr)
    }

    fn parse_key_value_pairs(&mut self, tokens: &TokenStream, attr: &Attribute) -> Result<(), Error> {
        let mut iter = tokens.clone().into_iter().peekable();
        let mut last_span = attr.span();

        loop {
            self.digest_key_value(&mut iter, &mut last_span)?;

            if iter.peek().is_none() {
                break;
            }

            self.digest_comma(&mut iter, &mut last_span)?;
        }

        Ok(())
    }

    fn digest_key_value(&mut self, iter: &mut TokenIter, last_span: &mut Span) -> Result<(), Error> {
        let (key, key_span) = self.digest_key(iter, last_span)?;
        self.digest_equal_sign(iter, &key, last_span)?;
        let value = self.digest_value(iter, &key, last_span)?;

        self.insert_with_duplicate_check(key, value, key_span)?;

        Ok(())
    }

    fn digest_key(&self, iter: &mut TokenIter, last_span: &mut Span)
        -> Result<(String, Span), Error>
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
            None => Err(Error::new(*last_span,
                "Expected identifier (key name), found end of tokens")),
        }
    }

    fn digest_equal_sign(
        &self,
        iter: &mut TokenIter,
        key: &str,
        last_span: &mut Span
    ) -> Result<(), Error> {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                *last_span = punct.span();
                Ok(())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other,
                    format!("Expected '=' after key '{}'", key)))
            }
            None => Err(Error::new(*last_span,
                format!("Expected '=' after key '{}'", key))),
        }
    }

    fn digest_value(
        &self,
        iter: &mut TokenIter,
        key: &str,
        last_span: &mut Span
    ) -> Result<String, Error> {
        match iter.next() {
            Some(TokenTree::Literal(lit)) => {
                *last_span = lit.span();
                let value_str = lit.to_string();

                if (value_str.starts_with('"') && value_str.ends_with('"')) ||
                   (value_str.starts_with('\'') && value_str.ends_with('\'')) {
                    Ok(value_str[1..value_str.len()-1].to_string())
                } else {
                    Ok(value_str)
                }
            }
            Some(TokenTree::Ident(ident)) => {
                *last_span = ident.span();
                Ok(ident.to_string())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other,
                    format!("Expected value after '{}='", key)))
            }
            None => Err(Error::new(*last_span,
                format!("Expected value after '{}='", key))),
        }
    }

    fn digest_comma(
        &mut self,
        iter: &mut TokenIter,
        last_span: &mut Span
    ) -> Result<(), Error> {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                *last_span = punct.span();
                Ok(())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other,
                    "Expected comma between key-value pairs"))
            }
            None => Err(Error::new(*last_span,
                "Expected comma, found end of tokens")),
        }
    }

    fn insert_with_duplicate_check(
        &mut self,
        argument_name: String,
        value: String,
        key_span: Span
    ) -> Result<(), Error> {
        if let Some(existing) = self.get(&argument_name) {
            let mut err1 = Error::new(
                key_span,
                format!("attribute value for '{}' specified more than once", argument_name),
            );
            let err2 = Error::new(
                existing.key_span,
                format!("first specification of value for '{}'", argument_name),
            );
            err1.combine(err2);
            return Err(err1);
        }

        self.insert(argument_name, value, key_span);

        Ok(())
    }
}
