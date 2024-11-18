use std::sync::Arc;

use async_stream::stream;
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::json;
use tabby_common::index::structured_doc::fields;
use tabby_inference::Embedding;
use tokio::task::JoinHandle;

use super::{build_tokens, BuildStructuredDoc};

pub struct IssueDocument {
    pub link: String,
    pub title: String,
    pub body: String,
    pub closed: bool,
}

#[async_trait]
impl BuildStructuredDoc for IssueDocument {
    fn should_skip(&self) -> bool {
        false
    }

    async fn build_attributes(&self) -> serde_json::Value {
        json!({
            fields::issue::LINK: self.link,
            fields::issue::TITLE: self.title,
            fields::issue::BODY: self.body,
            fields::issue::CLOSED: self.closed,
        })
    }

    async fn build_chunk_attributes(
        &self,
        embedding: Arc<dyn Embedding>,
    ) -> BoxStream<JoinHandle<(Vec<String>, serde_json::Value)>> {
        let text = format!("{}\n\n{}", self.title, self.body);
        let s = stream! {
            yield tokio::spawn(async move {
                let tokens = build_tokens(embedding, &text).await;
                let chunk_attributes = json!({});
                (tokens, chunk_attributes)
            })
        };

        Box::pin(s)
    }
}