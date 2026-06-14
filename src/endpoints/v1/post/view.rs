use crate::endpoints::v1::post::endpoint::CreateChatError;
use actix_web::web;
use utoipa::ToSchema;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateChatView {
    members: Vec<u64>,
    name: String,
}

impl CreateChatView {
    pub fn new(members: Vec<u64>, name: String) -> Self {
        Self { members, name }
    }

    pub fn members(&self) -> &[u64] {
        &self.members
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<web::Json<CreateChatView>> for CreateChatView {
    type Error = CreateChatError;

    fn try_from(params: web::Json<CreateChatView>) -> Result<CreateChatView, Self::Error> {
        Ok(params.into_inner())
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateChatResultView {
    id: u64,
    name: String,
}

impl CreateChatResultView {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
