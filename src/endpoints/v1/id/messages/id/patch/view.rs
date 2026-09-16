use actix_web::web;

use crate::endpoints::v1::id::messages::id::patch::endpoint::PatchMessageError;

/// Nouveau contenu d'un message, qui remplace entièrement l'ancien.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PatchMessageView {
    /// Nouveau contenu, qui remplace intégralement l'ancien. Obligatoire.
    #[schema(example = "La réunion est finalement décalée à 16h.")]
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
