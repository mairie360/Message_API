use crate::endpoints::health::HealthDoc;
use crate::endpoints::hello::HelloDoc;
use crate::endpoints::v1::doc::V1Doc;
use utoipa::OpenApi;

// Dans votre ApiDoc principale
#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = HelloDoc),
    )
)]
pub struct ApiDoc;
