use std::collections::HashMap;
use quote::ToTokens;
use syn::{Attribute, Error, ItemFn};

/// Parse all #[defaults(...)] attributes and merge them into a single HashMap.
/// Multiple #[defaults(...)] attributes are allowed and will be combined.
pub fn parse_defaults_attributes(func: &ItemFn) -> Result<HashMap<String, String>, Error> {
    let mut combined = HashMap::new();

    for attr in &func.attrs {
        if attr.path().is_ident("defaults") {
            let map = parse_single_defaults_attribute(attr)?;

            // Merge & detect duplicates across multiple attributes
            for (k, v) in map {
                if let Some(existing) = combined.get(&k) {
                    return Err(Error::new_spanned(
                        attr,
                        format!(
                            "Duplicate default value for argument '{}' (was: \"{}\", now: \"{}\")",
                            k, existing, v
                        ),
                    ));
                }
                combined.insert(k, v);
            }
        }
    }

    Ok(combined)
}

fn parse_single_defaults_attribute(attr: &Attribute) -> Result<HashMap<String, String>, Error> {
    let meta_list = attr.meta.require_list().map_err(|_| {
        Error::new_spanned(
            attr,
            "Invalid defaults attribute format. Expected: #[defaults(key=\"value\", ...)]",
        )
    })?;

    let tokens = meta_list.tokens.to_string();
    parse_key_value_list(&tokens, attr)
}

fn parse_key_value_list(
    input: &str,
    span_for_errors: &dyn ToTokens,
) -> Result<HashMap<String, String>, Error> {
    let mut map = HashMap::new();

    for pair in input.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| {
                Error::new_spanned(
                    span_for_errors,
                    format!("Invalid defaults syntax: '{}'. Expected key=\"value\"", pair),
                )
            })?;

        let key = key.trim();
        let value = value.trim();

        if !value.starts_with('"') || !value.ends_with('"') {
            return Err(Error::new_spanned(
                span_for_errors,
                format!("Default value for '{}' must be a quoted string", key),
            ));
        }

        let value_unquoted = value.trim_matches('"').to_string();

        if let Some(existing) = map.get(key) {
            return Err(Error::new_spanned(
                span_for_errors,
                format!(
                    "Duplicate default within attribute for '{}': \"{}\" vs \"{}\"",
                    key, existing, value_unquoted
                ),
            ));
        }

        map.insert(key.to_string(), value_unquoted);
    }

    Ok(map)
}
