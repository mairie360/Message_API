use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::id::messages::post::endpoint::PosteMessageError;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PostMessageView {
    sitation: Option<u64>, // message sitation
    content: String,
}

impl PostMessageView {
    pub fn new(sitation: Option<u64>, content: String) -> Self {
        Self { sitation, content }
    }

    pub fn sitation(&self) -> Option<u64> {
        self.sitation
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl TryFrom<web::Json<PostMessageView>> for PostMessageView {
    type Error = PosteMessageError;

    fn try_from(params: web::Json<PostMessageView>) -> Result<PostMessageView, Self::Error> {
        Ok(params.into_inner())
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PostMessageResultView {
    pub sitation: Option<u64>,
    pub content: String,
    pub id: u64,
}

impl PostMessageResultView {
    pub fn new(sitation: Option<u64>, content: String, id: u64) -> Self {
        Self {
            sitation,
            content,
            id,
        }
    }

    pub fn sitation(&self) -> Option<u64> {
        self.sitation
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
