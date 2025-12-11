use syn::Ident;

/// Convert an identifier to camel case (PascalCase)
/// Example: "my_function" -> "MyFunction"
pub fn convert_ident_to_camel_case(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let camel_case = s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();
    
    Ident::new(&camel_case, ident.span())
}

/// Convert a function name to an enum variant name
/// Example: "my_tusk_function" -> "MyTuskFunction"
pub fn convert_function_to_enum_variant(func_name: &Ident) -> Ident {
    convert_ident_to_camel_case(func_name)
}

/// Convert a submodule name to an enum variant name
/// Example: "sub_module" -> "SubModule"
pub fn convert_submodule_to_enum_variant(submod_name: &Ident) -> Ident {
    convert_ident_to_camel_case(submod_name)
}

/// Convert an external module name to an enum variant name
/// Example: "my_lib" -> "MyLib"
pub fn convert_external_module_to_enum_variant(alias: &Ident) -> Ident {
    convert_ident_to_camel_case(alias)
}
