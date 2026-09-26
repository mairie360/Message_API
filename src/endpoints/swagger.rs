use crate::endpoints::health::HealthDoc;
use crate::endpoints::hello::HelloDoc;
use crate::endpoints::v1::doc::V1Doc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Message API — Mairie 360",
        version = "1.0.0",
        description = "\
API de messagerie de la plateforme **Mairie 360** : conversations, messages, participants et flux \
temps réel. Les comptes et les rôles vivent dans Core API.

## Temps réel

Les opérations d'écriture sont de simples appels HTTP ; la diffusion aux autres participants passe \
par `GET /api/v1/stream`, un canal **SSE** (`text/event-stream`) que chaque client garde ouvert. \
Chaque ligne `data:` y porte un objet `ChatSignal` qui dit *qu'il s'est passé quelque chose* dans \
une conversation, sans transporter le message lui-même : le client recharge alors \
`GET /api/v1/{chat_id}/`.

Un commentaire `: ping` est émis toutes les 15 secondes pour tenir la connexion ouverte à travers \
les proxys ; les clients SSE l'ignorent d'eux-mêmes.

## Contrôle d'accès

Attention : hormis `GET /api/v1/`, qui ne liste que les conversations de l'appelant, **aucune \
opération ne vérifie que l'appelant est membre de la conversation visée**. Tout utilisateur \
authentifié peut lire, modifier ou supprimer n'importe quelle conversation dès lors qu'il en \
connaît l'identifiant. Ces routes ne renvoient donc jamais `403`.

## Error format

Error responses (`4xx` and `5xx`) have a **`text/plain`** body holding the error message, not a \
JSON object. Every response carries `X-Content-Type-Options: nosniff`.

Statuses returned across the API, before the handler runs:

| Status | Meaning |
| --- | --- |
| `400` | URL segment that is not an integer, malformed JSON body, or a field breaking its \
validation rules (length, control characters, `<` / `>`); the body names the first invalid \
field, e.g. ``Invalid `content`: must not contain `<` or `>` ``. |
| `401` | `Authorization` header missing or malformed, invalid or expired JWT, or revoked session. |
| `500` | Database or Redis failure. |
",
        contact(
            name = "Équipe Mairie 360",
            url = "https://github.com/mairie360"
        ),
        license(
            name = "Propriétaire",
            identifier = "LicenseRef-mairie360-proprietary"
        )
    ),
    servers(
        (url = "http://localhost:3003", description = "Développement local (cargo run)"),
        (url = "http://development.mairie360.fr", description = "Pile Docker de développement (nginx)")
    ),
    tags(
        (name = "Chats", description = "Conversations : liste, création, lecture des messages et suppression."),
        (name = "Messages", description = "Messages d'une conversation : publication, modification et suppression."),
        (name = "Users", description = "Participants d'une conversation : consultation, ajout et retrait."),
        (name = "Stream", description = "Canal SSE de notification temps réel."),
        (name = "Service", description = "Sondes techniques non authentifiées, utilisées par Docker et Kubernetes.")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = HelloDoc),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Sans ce modifier, les opérations qui déclarent `security(("jwt" = []))` référencent un schéma
/// absent du contrat : Swagger UI n'offre pas de bouton « Authorize » et les clients générés
/// pointent dans le vide.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT émis par Core API (`POST /api/v1/auth/login`)."))
                    .build(),
            ),
        )
    }
}
