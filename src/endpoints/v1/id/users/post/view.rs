use actix_web::web;
use utoipa::ToSchema;

use crate::endpoints::v1::id::users::post::endpoint::AddUsersToChatError;

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUsersToChat {
    pub users_id: Vec<u64>,
}

impl AddUsersToChat {
    pub fn new(users_id: Vec<u64>) -> Self {
        Self { users_id }
    }

    pub fn users_id(&self) -> &[u64] {
        &self.users_id
    }
}

impl TryFrom<web::Json<AddUsersToChat>> for AddUsersToChat {
    type Error = AddUsersToChatError;

    fn try_from(params: web::Json<AddUsersToChat>) -> Result<AddUsersToChat, Self::Error> {
        Ok(params.into_inner())
    }
}
