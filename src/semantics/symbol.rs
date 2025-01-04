use std::collections::HashMap;

#[derive(Debug)]
pub struct SyntaxError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub scope_path: Vec<String>, // e.g. ["module", "class_name", "function_name"]
}

#[derive(Debug, PartialEq)]
pub struct Reference {
    pub name: String,
    pub location: Location,
    pub scope_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub start: (usize, usize), // (line, column)
    pub end: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Variable,
    Class,
    Parameter,
    Module,
    Unknown,
    // TODO: add mores
}


