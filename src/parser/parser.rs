use log::debug;

use super::queries;
use super::symbol;

use streaming_iterator::StreamingIterator;

/// The main trait that defines the parser interface
pub trait Parser {
    /// Initialize the parser with necessary configuration
    fn new() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Parse the given source code and return a synstax tree
    fn parse(
        &mut self,
        source: &str,
    ) -> Result<(Vec<symbol::Symbol>, Vec<symbol::Reference>), Vec<symbol::SyntaxError>>;

    /// Get all syntax errors in the current tree
    fn _get_errors(&self) -> Vec<symbol::SyntaxError>;
}

/// Tree-sitter implementation of the Parser trait
pub struct TreeSitterParser {
    parser: tree_sitter::Parser,
}

impl Parser for TreeSitterParser {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

        Ok(Self { parser })
    }

    fn parse(
        &mut self,
        source: &str,
    ) -> Result<(Vec<symbol::Symbol>, Vec<symbol::Reference>), Vec<symbol::SyntaxError>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            vec![symbol::SyntaxError {
                message: "Failed to parse".to_string(),
                line: 0,
                column: 0,
            }]
        })?;

        // Collect symbols
        let mut collector = SymbolCollector::new(source.as_bytes());
        let errors = collector.collect_symbols(&tree);

        if let Err(e) = errors {
            debug!("Failed to collect symbols: {}", e);
            return Err(vec![symbol::SyntaxError {
                message: format!("Failed to collect symbols: {}", e),
                line: 0,
                column: 0,
            }]);
        }

        return Ok((collector.declarations, collector.references));
    }

    fn _get_errors(&self) -> Vec<symbol::SyntaxError> {
        // TODO: Implement error collection from tree-sitter
        Vec::new()
    }
}

/// A helper struct to convert tree-sitter nodes to perform
/// symbol collection and scope analysis
pub struct SymbolCollector<'a> {
    pub declarations: Vec<symbol::Symbol>,
    pub references: Vec<symbol::Reference>,

    source: &'a [u8],
}

impl<'a> SymbolCollector<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            declarations: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn collect_symbols(
        &mut self,
        tree: &tree_sitter::Tree,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let declaration_query = queries::get_declaration_query()
            .map_err(|e| format!("Failed to get declaration query: {}", e))?;

        // Collect declarations
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let mut matches = query_cursor.matches(&declaration_query, tree.root_node(), self.source);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let name = capture.node.utf8_text(self.source).unwrap().to_string();
                let location = self.get_location(capture.node);
                let scope_path = self.get_scope_path(capture.node);

                let kind = match capture.index as u32 {
                    0 => symbol::SymbolKind::Function,  // Function
                    1 => symbol::SymbolKind::Class,     // Class
                    2 => symbol::SymbolKind::Variable,  // Assignment
                    3 => symbol::SymbolKind::Parameter, // Parameters
                    4 => symbol::SymbolKind::Module,    // Module
                    _ => {
                        debug!("Unknown symbol kind: {}", capture.index);
                        symbol::SymbolKind::Unknown
                    }
                };

                self.declarations.push(symbol::Symbol {
                    name,
                    kind,
                    location,
                    scope_path,
                });
            }
        }

        // Collect references
        let reference_query = queries::get_reference_query()
            .map_err(|e| format!("Failed to get reference query: {}", e))?;

        let mut query_cursor = tree_sitter::QueryCursor::new();
        let mut matches = query_cursor.matches(&reference_query, tree.root_node(), self.source);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                // Skip nodes that are already captured as declarations
                if self.declarations.iter().any(|d| {
                    d.location.start == self.get_location(capture.node).start
                        && d.location.end == self.get_location(capture.node).end
                }) {
                    continue;
                }

                let name = capture.node.utf8_text(self.source).unwrap().to_string();
                let location = self.get_location(capture.node);
                let scope_path = self.get_scope_path(capture.node);

                self.references.push(symbol::Reference {
                    name,
                    location,
                    scope_path,
                });
            }
        }

        return Ok(());
    }

    fn get_location(&self, node: tree_sitter::Node) -> symbol::Location {
        symbol::Location {
            start: (node.start_position().row, node.start_position().column),
            end: (node.end_position().row, node.end_position().column),
        }
    }

    fn get_scope_path(&self, node: tree_sitter::Node) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = node.parent();

        while let Some(parent) = current {
            match parent.kind() {
                "function_definition" | "class_definition" => {
                    if let Some(name_node) = parent
                        .children(&mut parent.walk())
                        .find(|n| n.kind() == "identifier")
                    {
                        if let Ok(name) = name_node.utf8_text(self.source) {
                            path.push(name.to_string());
                        }
                    }
                }
                "module" => {
                    path.push("module".to_string());
                }
                _ => {}
            }
            current = parent.parent();
        }

        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbol::{Location, Symbol, SymbolKind};

    #[test]
    fn basic_variable_test() {
        let source_code = r#"
        x = 42
        "#;

        let mut parser = TreeSitterParser::new().unwrap();
        let (symbols, references) = parser.parse(source_code).unwrap();

        // Account for some extra indentation to the start column
        // to make the test code more readable
        let ident_length = 8;

        let expected_symbols = vec![Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                start: (1, ident_length),
                end: (1, 1 + ident_length),
            },
            scope_path: vec!["module".to_string()],
        }];

        assert_eq!(symbols, expected_symbols);
        assert!(references.is_empty());
    }

    #[test]
    fn basic_function_test() {
        let source_code = r#"
        def foo():
            pass
        "#;

        let mut parser = TreeSitterParser::new().unwrap();
        let (symbols, references) = parser.parse(source_code).unwrap();

        // Account for some extra indentation to the start column
        // to make the test code more readable
        let ident_length = 8;

        let expected_symbols = vec![Symbol {
            name: "foo".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                start: (1, 4 + ident_length),
                end: (1, 7 + ident_length),
            },
            scope_path: vec!["module".to_string(), "foo".to_string()],
        }];

        assert_eq!(symbols, expected_symbols);
        assert!(references.is_empty());
    }

    #[test]
    fn function_with_parameters() {
        let source_code = r#"
        def foo(a,b):
            return a + b
        "#;

        let mut parser = TreeSitterParser::new().unwrap();
        let (symbols, _) = parser.parse(source_code).unwrap();

        // Account for some extra indentation to the start column
        // to make the test code more readable
        let ident_length = 8;

        let expected_symbols = vec![
            Symbol {
                name: "foo".to_string(),
                kind: SymbolKind::Function,
                location: Location {
                    start: (1, 4 + ident_length),
                    end: (1, 7 + ident_length),
                },
                scope_path: vec!["module".to_string(), "foo".to_string()],
            },
            Symbol {
                name: "a".to_string(),
                kind: SymbolKind::Variable,
                location: Location {
                    start: (1, 8 + ident_length),
                    end: (1, 9 + ident_length),
                },
                scope_path: vec!["module".to_string(), "foo".to_string()],
            },
            Symbol {
                name: "b".to_string(),
                kind: SymbolKind::Variable,
                location: Location {
                    start: (1, 10 + ident_length),
                    end: (1, 11 + ident_length),
                },
                scope_path: vec!["module".to_string(), "foo".to_string()],
            },
        ];
        assert_eq!(symbols, expected_symbols);
    }
}
