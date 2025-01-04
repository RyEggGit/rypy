pub fn get_declaration_query() -> Result<tree_sitter::Query, tree_sitter::QueryError> {
    tree_sitter::Query::new(
        &tree_sitter_python::LANGUAGE.into(),
        r#"
        (function_definition
          name: (identifier) @function.def)
        (class_definition
          name: (identifier) @class.def)
        (assignment 
          left: (identifier) @variable.def)
        (parameters 
          (identifier) @variable.def)
    "#,
    )
}

pub fn get_reference_query() -> Result<tree_sitter::Query, tree_sitter::QueryError> {
    tree_sitter::Query::new(
        &tree_sitter_python::LANGUAGE.into(),
        r#"
        (identifier) @reference
    "#,
    )
}
