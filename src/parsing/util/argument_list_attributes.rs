use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{Error, ItemFn, Attribute, spanned::Spanned};

use crate::parsing::util::types::ArgumentListAttributes;

type TokenIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

impl ArgumentListAttributes {
    /// Parses list-style attributes from a function where arguments are listed
    /// without values, separated by commas.
    ///
    /// Each parsed identifier corresponds to the *name of a function argument*.
    /// After parsing, the macro validates that every identifier matches
    /// an actual argument of the function. Unknown argument names result
    /// in a precise, span-based compiler error.
    ///
    /// Each identifier is stored together with:
    /// - its span (used for error reporting)
    ///
    /// The order of identifiers is preserved in the Vec, which is important
    /// for attributes like `#[positional(...)]` where order matters.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[positional(required, user_id)]
    /// fn run(required: String, user_id: u32, verbose: bool) {}
    ///
    /// let attrs = ArgumentListAttributes::from_func(&func, "positional")?;
    ///
    /// assert!(attrs.contains("required"));
    /// assert_eq!(attrs.position("user_id"), Some(1));
    /// assert_eq!(attrs.len(), 2);
    /// ```
    ///
    /// This struct is intentionally designed so macros can:
    /// - validate identifiers *against actual function arguments*
    /// - detect duplicates
    /// - preserve ordering for positional attributes
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
                format!("Invalid {} attribute format. Expected: #[{}(arg1, arg2, ...)]",
                    attr_name, attr_name),
            )
        })?;

        self.parse_identifier_list(&meta_list.tokens, attr)
    }

    fn parse_identifier_list(&mut self, tokens: &TokenStream, attr: &Attribute) -> Result<(), Error> {
        let mut iter = tokens.clone().into_iter().peekable();
        let mut last_span = attr.span();

        // Handle empty attribute list
        if iter.peek().is_none() {
            return Ok(());
        }

        loop {
            self.digest_identifier(&mut iter, &mut last_span)?;

            if iter.peek().is_none() {
                break;
            }

            self.digest_comma(&mut iter, &mut last_span)?;
        }

        Ok(())
    }

    fn digest_identifier(&mut self, iter: &mut TokenIter, last_span: &mut Span) -> Result<(), Error> {
        match iter.next() {
            Some(TokenTree::Ident(ident)) => {
                let argument_name = ident.to_string();
                let span = ident.span();
                *last_span = span;

                self.push_with_duplicate_check(argument_name, span)?;

                Ok(())
            }
            Some(other) => {
                *last_span = other.span();
                Err(Error::new_spanned(other, "Expected identifier (argument name)"))
            }
            None => Err(Error::new(*last_span,
                "Expected identifier (argument name), found end of tokens")),
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
                    "Expected comma between identifiers"))
            }
            None => Err(Error::new(*last_span,
                "Expected comma, found end of tokens")),
        }
    }

    fn push_with_duplicate_check(
        &mut self,
        argument_name: String,
        span: Span
    ) -> Result<(), Error> {
        // Check for duplicates by searching through existing arguments
        if let Some(existing_pos) = self.position(&argument_name) {
            if let Some(existing) = self.get(existing_pos) {
                let mut err1 = Error::new(
                    span,
                    format!("argument '{}' specified more than once", argument_name),
                );
                let err2 = Error::new(
                    existing.span,
                    format!("first specification of '{}'", argument_name),
                );
                err1.combine(err2);
                return Err(err1);
            }
        }

        self.push(argument_name, span);

        Ok(())
    }
}
