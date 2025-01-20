use log::debug;

use crate::{
    lsp::document_sync::Position,
    parser::symbol::{self, Location},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReferenceGraph {
    // Map from symbol ID to the actual symbol
    definitions: HashMap<String, Vec<symbol::Symbol>>,

    references: Vec<symbol::Reference>,

    // Map from symbol ID to its references
    resolved_references: HashMap<String, symbol::Symbol>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            references: Vec::new(),
            resolved_references: HashMap::new(),
        }
    }

    /// Build the reference graph from collected definitions and references
    pub fn build(definitions: Vec<symbol::Symbol>, references: Vec<symbol::Reference>) -> Self {
        let mut graph = Self::new();

        // First, index all symbols by their unique ID (name + scope path)
        for definition in definitions {
            let id = graph.create_symbol_scope_id(&definition.name, &definition.scope_path);
            graph.definitions.entry(id).or_default().push(definition);
        }

        graph.references = references;

        // Then resolve each reference to its symbol
        for reference in &graph.references {
            if let Some(definition) = graph.resolve_reference(&reference) {
                let definition_id = graph.create_symbol_id(&reference.name, &reference.location);
                graph.resolved_references.insert(definition_id, definition);
            }
        }

        graph
    }

    /// Get symbol by position (this is going to be a slow operation for now
    /// until I think of a better way of implementing this. Likely moving the
    /// computation over to salsa)
    pub fn get_symbol_by_location(&self, position: Position) -> Option<&symbol::Reference> {
        for symbol in &self.references {
            let Location { start, end } = symbol.location;
            if start.0 <= position.line
                && end.0 >= position.line
                && start.1 <= position.character
                && end.1 >= position.character
            {
                return Some(symbol);
            }
        }
        None
    }

    /// Find the definition location for a symbol at the given position.
    /// Returns the Symbol if found, which contains the definition location.
    pub fn find_definition(&self, position: Position) -> Option<&symbol::Symbol> {
        // First get the symbol 
        let reference = self.get_symbol_by_location(position)?;

        // Get the symbols id
        let id = self.create_symbol_id(&reference.name, &reference.location);

        // Return the definition of the reference
        self.resolved_references.get(&id)
    }

    /// Create a ID for a symbol based on its name, scope path (! not unique)
    fn create_symbol_scope_id(&self, name: &str, scope_path: &[String]) -> String {
        format!("{}:{}", scope_path.join("/"), name)
    }

    /// Create a unique ID for a symbol based on its name and location of first character
    fn create_symbol_id(&self, name: &str, location: &Location) -> String {
        format!("{}:{}|{}", name, location.start.0, location.start.1)
    }

    /// Resolve a reference to its corresponding symbol by searching through scopes
    /// from most specific to most general, returning the best matching definition
    fn resolve_reference(&self, reference: &symbol::Reference) -> Option<symbol::Symbol> {
        // Start from the most specific scope and work outwards
        let mut current_scope = reference.scope_path.clone();

        while !current_scope.is_empty() {
            // Try to find the symbol in the current scope
            let definition_id = self.create_symbol_scope_id(&reference.name, &current_scope);

            if let Some(definitions) = self.definitions.get(&definition_id) {
                debug!(
                    "Found symbols in scope {:?}: {:?}",
                    current_scope, definitions
                );

                // Find the definition that is closest to the reference
                // by comparing their locations
                if let Some(best_definition) = definitions.iter().min_by_key(|d| {
                    // If the definition is before the reference, use the distance between them
                    // Otherwise, use a large value to prefer definitions that come before
                    if d.location.start.0 <= reference.location.start.0 {
                        reference.location.start.0 - d.location.start.0
                    } else {
                        std::usize::MAX
                    }
                }) {
                    return Some(best_definition.clone());
                }
            }

            // Move up one scope level if no suitable definition found
            current_scope.pop();
        }

        // Try global scope as last resort
        let global_id = self.create_symbol_scope_id(&reference.name, &["module".to_string()]);
        if let Some(definitions) = self.definitions.get(&global_id) {
            debug!("Found symbols in global scope: {:?}", definitions);
            return definitions
                .iter()
                .min_by_key(|d| {
                    if d.location.start.0 <= reference.location.start.0 {
                        reference.location.start.0 - d.location.start.0
                    } else {
                        std::usize::MAX
                    }
                })
                .cloned();
        }

        None
    }
}
