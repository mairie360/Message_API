use actix_web::web;

use crate::endpoints::v1::id::messages::id::patch::endpoint::PatchMessageError;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PatchMessageView {
    content: String,
}

impl PatchMessageView {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl TryFrom<web::Json<PatchMessageView>> for PatchMessageView {
    type Error = PatchMessageError;

    fn try_from(params: web::Json<PatchMessageView>) -> Result<PatchMessageView, Self::Error> {
        Ok(params.into_inner())
    }
}
