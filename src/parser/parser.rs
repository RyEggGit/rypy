use std::path::Path;

/// Represents a syntax error in the parsed code
#[derive(Debug)]
pub struct SyntaxError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Represents a position in the source code
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Represents a range in the source code
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Represents a node in the syntax tree
#[derive(Debug)]
pub struct SyntaxNode {
    pub kind: String,
    pub range: Range,
    pub children: Vec<SyntaxNode>,
}

/// The main trait that defines the parser interface
pub trait Parser {
    /// Initialize the parser with necessary configuration
    fn new() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Parse the given source code and return a syntax tree
    fn parse(&mut self, source: &str) -> Result<SyntaxNode, Vec<SyntaxError>>;

    /// Parse a file at the given path
    fn parse_file(&mut self, path: &Path) -> Result<SyntaxNode, Box<dyn std::error::Error>>;

    /// Update an existing syntax tree with changes
    fn update(
        &mut self,
        old_tree: Option<&SyntaxNode>,
        source: &str,
        edit_range: Range,
    ) -> Result<SyntaxNode, Vec<SyntaxError>>;

    /// Get all syntax errors in the current tree
    fn get_errors(&self) -> Vec<SyntaxError>;
}

/// Tree-sitter implementation of the Parser trait
pub struct TreeSitterParser {
    parser: tree_sitter::Parser,
    tree: Option<tree_sitter::Tree>,
}

impl Parser for TreeSitterParser {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

        Ok(Self { parser, tree: None })
    }

    fn parse(&mut self, source: &str) -> Result<SyntaxNode, Vec<SyntaxError>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            vec![SyntaxError {
                message: "Failed to parse".to_string(),
                line: 0,
                column: 0,
            }]
        })?;

        // Convert tree-sitter tree to our SyntaxNode structure
        // This is a simplified example - you'd want to implement proper conversion
        Ok(convert_tree_sitter_node(tree.root_node()))
    }

    fn parse_file(&mut self, path: &Path) -> Result<SyntaxNode, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content).map_err(|e| {
            let mut message = "Failed to parse file".to_string();
            for error in e {
                message.push_str(&format!(
                    "\n{} at {}:{}",
                    error.message, error.line, error.column
                ));
            }
            message.into()
        })
    }

    fn update(
        &mut self,
        _old_tree: Option<&SyntaxNode>,
        source: &str,
        _edit_range: Range,
    ) -> Result<SyntaxNode, Vec<SyntaxError>> {
        // TODO: implement incremental parsing
        self.parse(source)
    }

    fn get_errors(&self) -> Vec<SyntaxError> {
        // TODO: Implement error collection from tree-sitter
        Vec::new()
    }
}

// Helper function to convert tree-sitter nodes to our SyntaxNode structure
fn convert_tree_sitter_node(node: tree_sitter::Node) -> SyntaxNode {
    SyntaxNode {
        kind: node.kind().to_string(),
        range: Range {
            start: Position {
                line: node.start_position().row,
                column: node.start_position().column,
            },
            end: Position {
                line: node.end_position().row,
                column: node.end_position().column,
            },
        },
        children: node
            .children(&mut node.walk())
            .map(convert_tree_sitter_node)
            .collect(),
    }
}
