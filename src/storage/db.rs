use crate::{
    analysis::{self, reference::ReferenceGraph},
    parser::{
        parser::{Parser, TreeSitterParser},
        symbol::{Reference, Symbol},
    },
};
use salsa;
use std::sync::Arc;

#[salsa::query_group(StorageDatabase)]
pub trait Storage: salsa::Database {
    //  --------- Inputs ----------
    #[salsa::input]
    fn document_text(&self, uri: String) -> Option<Arc<String>>;

    #[salsa::input]
    fn document_version(&self, uri: String) -> Option<i32>;

    // ---------- Derived Queries --------------
    fn document_declaration(&self, uri: String) -> Option<Arc<(Vec<Symbol>, Vec<Reference>)>>;
    fn document_reference_graph(&self, uri: String) -> Option<ReferenceGraph>;
}

fn document_declaration(
    db: &dyn Storage,
    uri: String,
) -> Option<Arc<(Vec<Symbol>, Vec<Reference>)>> {
    let text = db.document_text(uri.clone())?;

    let mut parser: Box<dyn Parser> = match TreeSitterParser::new() {
        Ok(p) => Box::new(p),
        Err(_) => return None,
    };

    Some(Arc::new(parser.parse(&text).ok()?))
}

fn document_reference_graph(db: &dyn Storage, uri: String) -> Option<ReferenceGraph> {
    let (declarations, references) =
        Arc::try_unwrap(db.document_declaration(uri.clone())?).unwrap_or_else(|arc| (*arc).clone());
    let reference_graph = analysis::reference::ReferenceGraph::build(declarations, references);
    Some(reference_graph)
}

// Database implementation
#[derive(Default)]
#[salsa::database(StorageDatabase)]
pub struct LspDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for LspDatabase {}
