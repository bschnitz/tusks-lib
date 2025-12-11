use syn::{ItemStruct, ItemFn, Ident};

/// Represents a tusks module with all its elements
#[derive(Debug)]
pub struct TusksModule {
    /// Name of the module (e.g. "tasks", "sub1")
    pub name: Ident,

    /// The parent annotated as pub use ... as parent_
    pub external_parent: Option<ExternalModule>,
    
    /// The Parameters struct (if not existing during parse it will be generated)
    pub parameters: Option<TusksParameters>,
    
    /// List of all pub command functions
    pub tusks: Vec<Tusk>,
    
    /// List of all pub sub-modules (recursive)
    pub submodules: Vec<TusksModule>,
    
    /// List of all external modules (pub use ... as ...)
    pub external_modules: Vec<ExternalModule>,
}

/// Represents a parameters struct
pub struct TusksParameters {
    /// The underlying struct
    pub pstruct: ItemStruct,
}

/// Represents a command function (tusk)
pub struct Tusk {
    /// The underlying function
    pub func: ItemFn,
}

/// Represents an externally imported module
pub struct ExternalModule {
    /// The alias name (e.g. "sub2")
    pub alias: Ident,
    
    /// Span for error messages
    pub span: proc_macro2::Span,
}

impl std::fmt::Debug for TusksParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TusksParameters")
            .field("name", &self.pstruct.ident.to_string())
            .finish()
    }
}

impl std::fmt::Debug for Tusk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tusk")
            .field("name", &self.func.sig.ident.to_string())
            .finish()
    }
}

impl std::fmt::Debug for ExternalModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalModule")
            .field("alias", &self.alias.to_string())
            .finish()
    }
}
