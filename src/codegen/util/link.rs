use proc_macro2::TokenStream;

use crate::{LinkNode, TusksNode, codegen::util::misc::build_path_tokens};

impl LinkNode {
    /// Build a TokenStream like:
    ///   super::<parent.module_path[1..]>::<self.name>::__tusks_internal_module::<destination>
    /// It is meant to be called from the current __tusks_internal_module to reach a destination,
    /// e.g. a function in the link nodes __tusks_internal_module.
    pub fn build_internal_module_path(&self, parent: &TusksNode, destination: &str) -> TokenStream
    {
        let path = std::iter::empty()
            .chain(["super"])
            .chain(parent.relative_module_path())
            .chain([self.name.as_str(), "__tusks_internal_module", destination]);

        build_path_tokens(path)
    }
}
