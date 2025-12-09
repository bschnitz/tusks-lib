use clap::ValueHint;
use indexmap::IndexMap;
use syn::FnArg;

/// A tusks module or submodule which may contain of
/// - tusks (public functions in this module)
/// - childs (public submodules of this module)
/// - links (sumobudules refered to with a `pub use ...` statement)
#[derive(Debug)]
pub struct TusksNode {
    pub module_path: Vec<String>,
    pub is_link: bool,
    pub link_name: Option<String>,
    pub tusks: Vec<Tusk>,
    pub childs: Vec<TusksNode>,
    pub links: Vec<LinkNode>,
}

impl TusksNode {
    pub fn get_module_name(&self) -> &String {
        self.module_path.last().unwrap()
    }
}

/// This is just a reference to a module defined elsewhere with a `use ...` statement
#[derive(Debug)]
pub struct LinkNode {
    pub name: String,
}

/// A Tusk is essentially a public function in a tusks module.
#[derive(Debug)]
pub struct Tusk {
    pub name: String, // the function/tusk name
    pub arguments: IndexMap <String, Argument>, // argument name => argument
}

// Wrapper um ValueHint
#[derive(Debug, Clone)]
pub struct ValueHintWrapper(pub ValueHint);

/// Represents how often an argument can appear
#[derive(Debug)]
pub struct ArgumentMultiplicity {
    pub min: Option<usize>, // minimal number of values
    pub max: Option<usize>, // maximal number of values
}

pub struct NoDebug<T>(pub T);

impl<T> std::fmt::Debug for NoDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<skipped>")
    }
}

/// Represents a function argument (a "tusk")
#[derive(Debug)]
pub struct Argument {
    pub name: String,                  // name of the argument
    pub type_: String,                 // Rust type as string, used for value_parser
    pub default: Option<String>,       // default value
    pub optional: bool,                // is Option<T>?
    pub flag: bool,                    // is a boolean flag?
    pub positional: bool,              // positional argument
    pub count: Option<ArgumentMultiplicity>, // multiplicity (num_args)
    pub short: Option<char>,           // short name, e.g., '-f'
    pub help: Option<String>,          // help text
    pub hidden: bool,                  // hidden from help
    pub value_hint: Option<ValueHintWrapper>, // autocomplete hints
    pub arg_enum: Option<Vec<String>>, // possible enum values
    pub validator: Option<String>,     // path to validator function
    pub arg: Option<NoDebug<FnArg>>,
}
