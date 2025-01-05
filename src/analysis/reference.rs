use crate::{
    lsp::document_sync::Position,
    parser::symbol::{self, Location},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReferenceGraph {
    // Map from symbol ID to the actual symbol
    symbols: HashMap<String, symbol::Symbol>,
    // Map from symbol ID to its references
    references: HashMap<String, Vec<symbol::Reference>>,
}

impl ReferenceGraph {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            references: HashMap::new(),
        }
    }

    /// Build the reference graph from collected symbols and references
    pub fn build(symbols: Vec<symbol::Symbol>, references: Vec<symbol::Reference>) -> Self {
        let mut graph = Self::new();

        // First, index all symbols by their unique ID (name + scope path)
        for symbol in symbols {
            let id = graph.create_symbol_id(&symbol.name, &symbol.scope_path);
            graph.symbols.insert(id, symbol);
        }

        // Then resolve each reference to its symbol
        for reference in references {
            if let Some(symbol_id) = graph.resolve_reference(&reference) {
                graph
                    .references
                    .entry(symbol_id)
                    .or_default()
                    .push(reference);
            }
        }

        graph
    }

    /// Get all references to a symbol
    pub fn _get_references(&self, symbol_id: &str) -> Vec<&symbol::Reference> {
        self.references
            .get(symbol_id)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Get a symbol by its ID
    pub fn get_symbol_by_id(&self, symbol_id: &str) -> Option<&symbol::Symbol> {
        self.symbols.get(symbol_id)
    }

    /// Get symbol by position (this is going to be a slow operation for now
    /// until I think of a better way of implementing this. Likely moving the
    /// computation over to salsa)
    pub fn get_symbol_by_location(&self, position: Position) -> Option<&symbol::Symbol> {
        for symbol in self.symbols.values() {
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
        // First check if we're directly on a symbol definition
        if let Some(symbol) = self.get_symbol_by_location(position.clone()) {
            return Some(symbol);
        }

        // Check all references to find one at this position
        for (symbol_id, references) in &self.references {
            for reference in references {
                let Location { start, end } = reference.location;
                // Check if position is within this reference
                if start.0 <= position.line 
                    && end.0 >= position.line
                    && start.1 <= position.character
                    && end.1 >= position.character 
                {
                    // Found a reference at this position, return its symbol
                    return self.get_symbol_by_id(symbol_id);
                }
            }
        }

        None
    }

    /// Create a unique ID for a symbol based on its name and scope path
    fn create_symbol_id(&self, name: &str, scope_path: &[String]) -> String {
        format!("{}:{}", scope_path.join("::"), name)
    }

    /// Resolve a reference to its corresponding symbol
    fn resolve_reference(&self, reference: &symbol::Reference) -> Option<String> {
        // Start from the most specific scope and work outwards
        let mut current_scope = reference.scope_path.clone();

        while !current_scope.is_empty() {
            // Try to find the symbol in the current scope
            let potential_id = self.create_symbol_id(&reference.name, &current_scope);
            if self.symbols.contains_key(&potential_id) {
                return Some(potential_id);
            }

            // Move up one scope level
            current_scope.pop();
        }

        // Try global scope as last resort
        let global_id = self.create_symbol_id(&reference.name, &["module".to_string()]);
        if self.symbols.contains_key(&global_id) {
            Some(global_id)
        } else {
            None
        }
    }
}
