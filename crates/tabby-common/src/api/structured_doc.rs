use async_trait::async_trait;
use tantivy::{
    schema::{self, document::CompactDocValue, Value},
    TantivyDocument,
};
use thiserror::Error;

use crate::index::{structured_doc, IndexSchema};

pub struct DocSearchResponse {
    pub hits: Vec<DocSearchHit>,
}

pub struct DocSearchHit {
    pub score: f32,
    pub doc: DocSearchDocument,
}

#[derive(Clone)]
pub enum DocSearchDocument {
    Web(DocSearchWebDocument),
    Issue(DocSearchIssueDocument),
}

#[derive(Error, Debug)]
pub enum DocSearchError {
    #[error("index not ready")]
    NotReady,

    #[error(transparent)]
    QueryParserError(#[from] tantivy::query::QueryParserError),

    #[error(transparent)]
    TantivyError(#[from] tantivy::TantivyError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait DocSearch: Send + Sync {
    /// Search docs from underlying index.
    ///
    /// * `source_ids`: Filter documents by source IDs, when empty, search all sources.
    async fn search(
        &self,
        source_ids: &[String],
        q: &str,
        limit: usize,
    ) -> Result<DocSearchResponse, DocSearchError>;
}

#[derive(Clone)]
pub struct DocSearchWebDocument {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Clone)]
pub struct DocSearchIssueDocument {
    pub title: String,
    pub link: String,
    pub body: String,
    pub closed: bool,
}

pub trait FromTantivyDocument {
    fn from_tantivy_document(doc: &TantivyDocument, chunk: &TantivyDocument) -> Option<Self>
    where
        Self: Sized;
}

impl FromTantivyDocument for DocSearchDocument {
    fn from_tantivy_document(doc: &TantivyDocument, chunk: &TantivyDocument) -> Option<Self> {
        let schema = IndexSchema::instance();
        let kind = get_json_text_field(doc, schema.field_attributes, structured_doc::fields::KIND);

        match kind {
            "web" => {
                DocSearchWebDocument::from_tantivy_document(doc, chunk).map(DocSearchDocument::Web)
            }
            "issue" => DocSearchIssueDocument::from_tantivy_document(doc, chunk)
                .map(DocSearchDocument::Issue),
            _ => None,
        }
    }
}

impl FromTantivyDocument for DocSearchWebDocument {
    fn from_tantivy_document(doc: &TantivyDocument, chunk: &TantivyDocument) -> Option<Self> {
        let schema = IndexSchema::instance();
        let title = get_json_text_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::web::TITLE,
        );
        let link = get_json_text_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::web::LINK,
        );
        let snippet = get_json_text_field(
            chunk,
            schema.field_chunk_attributes,
            structured_doc::fields::web::CHUNK_TEXT,
        );

        Some(Self {
            title: title.into(),
            link: link.into(),
            snippet: snippet.into(),
        })
    }
}

impl FromTantivyDocument for DocSearchIssueDocument {
    fn from_tantivy_document(doc: &TantivyDocument, _: &TantivyDocument) -> Option<Self> {
        let schema = IndexSchema::instance();
        let title = get_json_text_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::issue::TITLE,
        );
        let link = get_json_text_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::issue::LINK,
        );
        let body = get_json_text_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::issue::BODY,
        );
        let closed = get_json_bool_field(
            doc,
            schema.field_attributes,
            structured_doc::fields::issue::CLOSED,
        );
        Some(Self {
            title: title.into(),
            link: link.into(),
            body: body.into(),
            closed,
        })
    }
}

fn get_json_field<'a>(
    doc: &'a TantivyDocument,
    field: schema::Field,
    name: &str,
) -> CompactDocValue<'a> {
    doc.get_first(field)
        .unwrap()
        .as_object()
        .unwrap()
        .find(|(k, _)| *k == name)
        .unwrap()
        .1
}

fn get_json_bool_field(doc: &TantivyDocument, field: schema::Field, name: &str) -> bool {
    get_json_field(doc, field, name).as_bool().unwrap()
}

fn get_json_text_field<'a>(doc: &'a TantivyDocument, field: schema::Field, name: &str) -> &'a str {
    get_json_field(doc, field, name).as_str().unwrap()
}
