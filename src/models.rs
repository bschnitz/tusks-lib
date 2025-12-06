use std::collections::HashMap;

/// A tusks module or submodule which may contain of
/// - tusks (public functions in this module)
/// - childs (public submodules of this module)
/// - links (sumobudules refered to with a `use ...` statement)
#[derive(Debug)]
pub struct TusksNode {
    pub module_name: String,
    pub tusks: Vec<Tusk>,
    pub childs: Vec<TusksNode>,
    pub links: Vec<LinkNode>,
}

/// This is just a reference to a module defined elsewhere with a `use ...` statement
#[derive(Debug)]
pub struct LinkNode {
    pub module_path: Vec<String>,
}

/// A Tusk is essentially a public function in a tusks module.
#[derive(Debug)]
pub struct Tusk {
    pub name: String, // the function/tusk name
    pub arguments: HashMap<String, Argument>, // argument name => argument
}

/// Essentially an argument of a function representing a tusk
/// NOTE:
/// - An argument is considered optional, if its type is Option<...>
/// - If an argument has a default value, it cannot be optional by implementation
/// This may seem a bit counterintuitive, but it simplifies the implementation and usage
#[derive(Debug)]
pub struct Argument {
    pub name: String, // name of the function argument
    pub type_: String, // rust type as string (just for cli-Docu)
    pub default: Option<String>, // default values for the argument
    pub optional: bool, // is the argument an optional argument
    pub value: Option<String>, // The actual value if already set
}
