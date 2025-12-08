use std::collections::HashMap;
use proc_macro2::Span;

/// A collection of parsed attributes for a function, organized by attribute name.
///
/// This structure stores multiple attribute sets (e.g., `defaults`, `validators`, `positional`),
/// where each attribute set can be either key-value pairs or a list of identifiers.
///
/// # Examples
///
/// ```rust
/// #[defaults(user_id = 42, verbose = false)]
/// #[validators(user_id = "positive", name = "non_empty")]
/// #[positional(required, user_id)]
/// fn run(required: String, user_id: u32, verbose: bool, name: String) {}
///
/// let attrs = FunctionAttributes::from_func(&func)?;
/// 
/// // Access key-value attributes
/// assert_eq!(attrs.get_key_value_attribute("defaults", "user_id").unwrap().value, "42");
/// 
/// // Access list attributes
/// let positional = attrs.get_list_set("positional").unwrap();
/// assert!(positional.contains("required"));
/// assert_eq!(positional.position("user_id"), Some(1));
/// ```
#[derive(Debug, Clone)]
pub struct FunctionAttributes {
    /// Maps attribute names to their parsed attribute data
    attribute_sets: HashMap<String, AttributeSet>,
}

/// Represents different types of attribute formats that can be parsed from function attributes.
///
/// This enum distinguishes between two common attribute patterns:
/// - Key-value pairs for configuration: `#[defaults(x = 1, y = "foo")]`
/// - Lists of identifiers for flags or ordering: `#[positional(a, b, c)]`
#[derive(Debug, Clone)]
pub enum AttributeSet {
    /// Key-value pairs: `#[defaults(user_id = 42, verbose = false)]`
    KeyValue(ArgumentKeyValueAttributes),
    /// List of identifiers: `#[positional(required, user_id)]`
    List(ArgumentListAttributes),
}

/// A collection of parsed attribute key/value pairs for a function.
///
/// This structure is used by procedural macros to extract and store
/// attribute metadata of the form:
///
/// ```rust
/// #[defaults(user_id = 42, verbose = false)]
/// fn run(required: String, user_id: u32, verbose: bool) {}
/// ```
///
/// The structure uses a HashMap internally, so the order of attributes is not preserved.
#[derive(Debug, Clone)]
pub struct ArgumentKeyValueAttributes {
    /// Maps argument names to their attribute values and spans
    attributes: HashMap<String, ArgumentKeyValueAttribute>,
}

/// A collection of argument names without values (list-style attributes).
///
/// This structure is used for attributes that specify a list of identifiers
/// without associated values, such as:
///
/// ```rust
/// #[positional(required, user_id)]
/// fn run(required: String, user_id: u32, verbose: bool) {}
/// ```
///
/// The order of arguments is preserved using a Vec, which is important for
/// positional or ordered attributes.
#[derive(Debug, Clone)]
pub struct ArgumentListAttributes {
    /// Ordered list of argument names with their spans
    arguments: Vec<ArgumentListAttribute>,
}

/// Represents a single key-value attribute for a function argument.
///
/// Contains the argument name, its value as a string, and the source code span
/// for error reporting.
#[derive(Debug, Clone)]
pub struct ArgumentKeyValueAttribute {
    pub argument_name: String,
    pub value: String,
    pub key_span: Span,
}

/// Represents a single argument name in a list-style attribute.
///
/// Contains the argument name and its source code span for error reporting.
#[derive(Debug, Clone)]
pub struct ArgumentListAttribute {
    pub argument_name: String,
    pub span: Span,
}

impl FunctionAttributes {
    /// Creates a new empty collection of function attributes.
    pub fn new() -> Self {
        Self {
            attribute_sets: HashMap::new(),
        }
    }

    /// Inserts a complete key-value attribute set for a given attribute name.
    ///
    /// This replaces any existing attribute set with the same name.
    ///
    /// # Example
    /// ```rust
    /// let mut attrs = FunctionAttributes::new();
    /// let mut defaults = ArgumentKeyValueAttributes::new();
    /// defaults.insert("x".to_string(), "42".to_string(), Span::call_site());
    /// attrs.insert_key_value_set("defaults".to_string(), defaults);
    /// ```
    pub fn insert_key_value_set(&mut self, attribute_name: String, attrs: ArgumentKeyValueAttributes) {
        self.attribute_sets.insert(attribute_name, AttributeSet::KeyValue(attrs));
    }

    /// Inserts a complete list attribute set for a given attribute name.
    ///
    /// This replaces any existing attribute set with the same name.
    pub fn insert_list_set(&mut self, attribute_name: String, attrs: ArgumentListAttributes) {
        self.attribute_sets.insert(attribute_name, AttributeSet::List(attrs));
    }

    /// Gets an attribute set by name, regardless of its type.
    ///
    /// Returns `None` if no attribute set with the given name exists.
    pub fn get_set(&self, attribute_name: &str) -> Option<&AttributeSet> {
        self.attribute_sets.get(attribute_name)
    }

    /// Gets a key-value attribute set by name.
    ///
    /// Returns `None` if the attribute set doesn't exist or is not a key-value set.
    pub fn get_key_value_set(&self, attribute_name: &str) -> Option<&ArgumentKeyValueAttributes> {
        match self.attribute_sets.get(attribute_name) {
            Some(AttributeSet::KeyValue(attrs)) => Some(attrs),
            _ => None,
        }
    }

    /// Gets a list attribute set by name.
    ///
    /// Returns `None` if the attribute set doesn't exist or is not a list set.
    pub fn get_list_set(&self, attribute_name: &str) -> Option<&ArgumentListAttributes> {
        match self.attribute_sets.get(attribute_name) {
            Some(AttributeSet::List(attrs)) => Some(attrs),
            _ => None,
        }
    }

    /// Gets or creates a key-value attribute set for the given attribute name.
    ///
    /// # Panics
    /// Panics if an attribute set with this name already exists but is not a key-value set.
    pub fn get_or_create_key_value_set(&mut self, attribute_name: &str) -> &mut ArgumentKeyValueAttributes {
        let entry = self.attribute_sets
            .entry(attribute_name.to_string())
            .or_insert_with(|| AttributeSet::KeyValue(ArgumentKeyValueAttributes::new()));
        
        match entry {
            AttributeSet::KeyValue(attrs) => attrs,
            _ => panic!("Attribute '{}' exists but is not a key-value set", attribute_name),
        }
    }

    /// Gets or creates a list attribute set for the given attribute name.
    ///
    /// # Panics
    /// Panics if an attribute set with this name already exists but is not a list set.
    pub fn get_or_create_list_set(&mut self, attribute_name: &str) -> &mut ArgumentListAttributes {
        let entry = self.attribute_sets
            .entry(attribute_name.to_string())
            .or_insert_with(|| AttributeSet::List(ArgumentListAttributes::new()));
        
        match entry {
            AttributeSet::List(attrs) => attrs,
            _ => panic!("Attribute '{}' exists but is not a list set", attribute_name),
        }
    }

    /// Convenience method to get a specific key-value attribute by attribute name and argument name.
    ///
    /// Returns `None` if the attribute set doesn't exist, is not a key-value set,
    /// or doesn't contain the specified argument.
    pub fn get_key_value_attribute(&self, attribute_name: &str, argument_name: &str) -> Option<&ArgumentKeyValueAttribute> {
        self.get_key_value_set(attribute_name)
            .and_then(|set| set.get(argument_name))
    }

    /// Inserts a single key-value attribute into the specified attribute set.
    ///
    /// Creates the attribute set if it doesn't exist.
    ///
    /// # Panics
    /// Panics if an attribute set with this name already exists but is not a key-value set.
    pub fn insert_key_value(&mut self, attribute_name: &str, argument_name: String, value: String, key_span: Span) {
        self.get_or_create_key_value_set(attribute_name)
            .insert(argument_name, value, key_span);
    }

    /// Inserts a single argument name into a list attribute set.
    ///
    /// Creates the attribute set if it doesn't exist.
    ///
    /// # Panics
    /// Panics if an attribute set with this name already exists but is not a list set.
    pub fn insert_list_item(&mut self, attribute_name: &str, argument_name: String, span: Span) {
        self.get_or_create_list_set(attribute_name)
            .push(argument_name, span);
    }

    /// Checks if an attribute set with the given name exists.
    pub fn has_attribute_set(&self, attribute_name: &str) -> bool {
        self.attribute_sets.contains_key(attribute_name)
    }

    /// Returns an iterator over all attribute sets with their names.
    pub fn iter_sets(&self) -> impl Iterator<Item = (&String, &AttributeSet)> {
        self.attribute_sets.iter()
    }

    /// Returns an iterator over all attribute set names.
    pub fn attribute_names(&self) -> impl Iterator<Item = &String> {
        self.attribute_sets.keys()
    }
}

impl ArgumentKeyValueAttributes {
    /// Creates a new empty collection of key-value attributes.
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Inserts a key-value attribute for a specific argument.
    ///
    /// If an attribute with the same argument name already exists, it will be replaced.
    pub fn insert(&mut self, argument_name: String, value: String, key_span: Span) {
        self.attributes.insert(
            argument_name.clone(),
            ArgumentKeyValueAttribute {
                argument_name,
                value,
                key_span,
            },
        );
    }

    /// Gets an attribute by argument name.
    ///
    /// Returns `None` if no attribute exists for this argument.
    pub fn get(&self, key: &str) -> Option<&ArgumentKeyValueAttribute> {
        self.attributes.get(key)
    }

    /// Checks if an attribute exists for the given argument name.
    pub fn contains_key(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Returns an iterator over all key-value attributes.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ArgumentKeyValueAttribute)> {
        self.attributes.iter()
    }

    /// Clones the internal HashMap and returns it.
    ///
    /// This is useful when you need owned data for further processing.
    pub fn to_map(&self) -> HashMap<String, ArgumentKeyValueAttribute> {
        self.attributes.clone()
    }

    /// Returns an iterator over all argument names that have attributes.
    pub fn argument_names(&self) -> impl Iterator<Item = &String> {
        self.attributes.keys()
    }
}

impl ArgumentListAttributes {
    /// Creates a new empty list of argument attributes.
    pub fn new() -> Self {
        Self {
            arguments: Vec::new(),
        }
    }

    /// Appends an argument name to the list.
    ///
    /// The order of insertion is preserved, which is important for positional attributes.
    pub fn push(&mut self, argument_name: String, span: Span) {
        self.arguments.push(ArgumentListAttribute {
            argument_name,
            span,
        });
    }

    /// Checks if the list contains an argument with the given name.
    pub fn contains(&self, argument_name: &str) -> bool {
        self.arguments.iter().any(|attr| attr.argument_name == argument_name)
    }

    /// Returns the position (0-indexed) of an argument in the list.
    ///
    /// Returns `None` if the argument is not in the list.
    pub fn position(&self, argument_name: &str) -> Option<usize> {
        self.arguments.iter().position(|attr| attr.argument_name == argument_name)
    }

    /// Gets an argument by its position in the list.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&ArgumentListAttribute> {
        self.arguments.get(index)
    }

    /// Returns an iterator over all arguments in the list.
    pub fn iter(&self) -> impl Iterator<Item = &ArgumentListAttribute> {
        self.arguments.iter()
    }

    /// Returns the number of arguments in the list.
    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    /// Returns an iterator over all argument names in order.
    pub fn argument_names(&self) -> impl Iterator<Item = &String> {
        self.arguments.iter().map(|attr| &attr.argument_name)
    }
}

impl Default for FunctionAttributes {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ArgumentKeyValueAttributes {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ArgumentListAttributes {
    fn default() -> Self {
        Self::new()
    }
}

// Beispiel-Nutzung:
#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn example_usage() {
        let mut attrs = FunctionAttributes::new();
        
        // Key-Value Attribute: #[defaults(user_id = 42, verbose = false)]
        attrs.insert_key_value("defaults", "user_id".to_string(), "42".to_string(), Span::call_site());
        attrs.insert_key_value("defaults", "verbose".to_string(), "false".to_string(), Span::call_site());
        
        // List Attribute: #[positional(required, user_id)]
        attrs.insert_list_item("positional", "required".to_string(), Span::call_site());
        attrs.insert_list_item("positional", "user_id".to_string(), Span::call_site());
        
        // Zugriff
        if let Some(list) = attrs.get_list_set("positional") {
            assert_eq!(list.len(), 2);
            assert!(list.contains("required"));
            assert_eq!(list.position("user_id"), Some(1));
        }
        
        if let Some(kv) = attrs.get_key_value_attribute("defaults", "user_id") {
            assert_eq!(kv.value, "42");
        }
    }
}
