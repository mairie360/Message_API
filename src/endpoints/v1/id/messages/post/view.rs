use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::id::messages::post::endpoint::PosteMessageError;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PostMessageView {
    /// Identifiant du message auquel celui-ci répond. Facultatif.
    #[schema(example = 100)]
    sitation: Option<u64>, // message sitation
    /// Contenu du message.
    #[schema(example = "La réunion est décalée à 15h.")]
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
    /// Identifiant attribué au message publié.
    #[schema(example = 101)]
    id: u64,
}

impl PostMessageResultView {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
