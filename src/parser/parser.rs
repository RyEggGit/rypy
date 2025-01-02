use log::debug;
use salsa::debug;
use serde::de;
use tree_sitter::{QueryMatch, QueryMatches};

use super::queries;
use crate::semantics::symbol::{self};

use streaming_iterator::StreamingIterator;

/// The main trait that defines the parser interface
pub trait Parser {
    /// Initialize the parser with necessary configuration
    fn new() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Parse the given source code and return a synstax tree
    fn parse(&mut self, source: &str) -> Result<Vec<symbol::Symbol>, Vec<symbol::SyntaxError>>;

    /// Get all syntax errors in the current tree
    fn get_errors(&self) -> Vec<symbol::SyntaxError>;
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

    fn parse(&mut self, source: &str) -> Result<Vec<symbol::Symbol>, Vec<symbol::SyntaxError>> {
        let tree = self.parser.parse(source, None).ok_or_else(|| {
            vec![symbol::SyntaxError {
                message: "Failed to parse".to_string(),
                line: 0,
                column: 0,
            }]
        })?;

        // Collect symbols and references
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

        debug!("Declarations: {:#?}", collector.declarations);

        return Ok(collector.declarations);
    }

    fn get_errors(&self) -> Vec<symbol::SyntaxError> {
        // TODO: Implement error collection from tree-sitter
        Vec::new()
    }
}

/// A helper struct to convert tree-sitter nodes to perform
/// symbol collection and scope analysis

pub struct SymbolCollector<'a> {
    pub declarations: Vec<symbol::Symbol>,

    source: &'a [u8],
}

impl<'a> SymbolCollector<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            declarations: Vec::new(),
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
            for capture in m.captures.iter() {
                let name = capture.node.utf8_text(self.source).unwrap().to_string();
                let location = self.get_location(capture.node);
                let scope_path = self.get_scope_path(capture.node);

                let kind = match capture.index as u32 {
                    0 => symbol::SymbolKind::Function, // Function
                    1 => symbol::SymbolKind::Class,    // Class
                    2 => symbol::SymbolKind::Variable, // Assignment
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
