#[derive(Debug)]
pub struct SyntaxError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub scope_path: Vec<String>, // e.g. ["module", "class_name", "function_name"]
}

#[derive(Debug)]
pub struct Reference {
    pub name: String,
    pub location: Location,
    pub scope_path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub start: (usize, usize), // (line, column)
    pub end: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Variable,
    Class,
    Parameter,
    Unknown,
    // TODO: add mores
}

