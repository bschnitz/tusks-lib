/// Context passed to build_* functions to make variable usage explicit
pub struct BuildContext<'a> {
    pub matches_var: &'a syn::Ident,
    pub link_path_var: &'a syn::Ident,
    pub path_sep: &'a str,
}
