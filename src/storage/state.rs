use log::error;

use super::db::{LspDatabase, Storage};
use crate::lsp::{
    document_sync::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        Position, Range,
    },
    language_features::{self, GotoDefinitionParams},
};
use std::sync::Arc;

pub struct LspState {
    db: LspDatabase,
    opened_uri: Option<String>,
}

impl LspState {
    pub fn new() -> Self {
        Self {
            opened_uri: None,
            db: LspDatabase::default(),
        }
    }

    pub fn open_document(&mut self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.db
            .set_document_text(uri.clone(), Some(Arc::new(params.text_document.text)));
        self.db
            .set_document_version(uri.clone(), Some(params.text_document.version));

        self.opened_uri = Some(uri);
    }

    pub fn _update_document(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.db
                .set_document_text(uri.clone(), Some(Arc::new(change.text)));
            self.db
                .set_document_version(uri, Some(params.text_document.version));
        }
    }

    pub fn _close_document(&mut self, _params: DidCloseTextDocumentParams) {
        self.opened_uri = None;
    }

    pub fn get_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> Option<language_features::Location> {
        let uri = match &self.opened_uri {
            Some(uri) => uri,
            None => {
                error!("Go to defintion was called without an opened uri");
                return None;
            }
        };

        // Get the reference graph that should be already computed
        let reference_graph = self.db.document_reference_graph(uri.clone())?;

        // Get the symbol's definiton
        let symbol_definition =
            reference_graph.find_definition(params.position)?;

        // Create the response
        let location = language_features::Location {
            uri: uri.to_string(),
            range: Range {
                start: Position {
                    line: symbol_definition.location.start.0,
                    character: symbol_definition.location.start.1,
                },
                end: Position {
                    line: symbol_definition.location.end.0,
                    character: symbol_definition.location.end.1,
                },
            },
        };

        Some(location)
    }
}
