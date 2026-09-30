use std::time::Duration;

use crate::sse::state::{AppState, ChatSignal};
use actix_web::{get, web, HttpResponse, Responder};
use mairie360_api_lib::security::AuthenticatedUser;
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio_stream::StreamExt;

#[utoipa::path(
    get,
    path = "",
    summary = "Ouvrir le flux de notifications temps réel",
    description = "Ouvre un canal **SSE** (`text/event-stream`) que le client garde ouvert pour \
                   être averti de l'activité de ses conversations, sans interroger l'API en \
                   boucle.\n\n\
                   Chaque ligne `data:` porte un objet `ChatSignal` indiquant *qu'il s'est passé \
                   quelque chose* dans une conversation — le contenu du message n'y est pas : le \
                   client recharge alors `GET /api/v1/{chat_id}/`.\n\n\
                   Un commentaire `: ping` est émis toutes les 15 secondes pour tenir la connexion \
                   ouverte à travers les proxys ; les clients SSE l'ignorent d'eux-mêmes. Le flux \
                   ne se termine pas de lui-même : c'est au client de se reconnecter s'il est \
                   coupé.\n\n\
                   Plusieurs flux peuvent être ouverts en parallèle pour un même utilisateur (onglets, \
                   appareils) : chacun reçoit les signaux, et la fermeture de l'un n'affecte pas les \
                   autres. Le `body` documenté ci-dessous \
                   décrit la charge utile d'**un** événement, pas la réponse entière.",
    responses(
        (
            status = 200,
            description = "Flux SSE établi. La connexion reste ouverte ; chaque ligne `data:` porte un `ChatSignal`.",
            content_type = "text/event-stream",
            body = ChatSignal,
            example = json!({ "type": "NEW_MSG", "chat_id": 5 })
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Stream",
)]
#[get("/stream")]
async fn sse_stream_route(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    // 1. Récupération de l'ID de l'agent connecté
    let user_id = auth_user.id;

    // 2. One channel per connection: a user may have several streams open at once.
    let (tx, rx) = mpsc::channel::<Result<actix_web::web::Bytes, String>>(10);

    // 3. Register this connection without replacing the user's other ones.
    let connection_id = state.register_connection(user_id, tx.clone());

    // 4. Keep-alive ping every 15 seconds; removes only this connection once it is dead.
    let state_clone = state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));

        // The first tick fires immediately, consume it.
        interval.tick().await;

        loop {
            tokio::select! {
                _ = tx.closed() => break,
                _ = interval.tick() => {
                    let ping_bytes = actix_web::web::Bytes::from(": ping\n\n");
                    // A full buffer is not a dead client: only a closed channel ends the stream.
                    if let Err(TrySendError::Closed(_)) = tx.try_send(Ok(ping_bytes)) {
                        break;
                    }
                }
            }
        }

        state_clone.remove_connection(user_id, connection_id);
    });

    // 5. Transformation du Receiver de Tokio en Stream Actix-web
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|result| match result {
        Ok(bytes) => Ok::<actix_web::web::Bytes, actix_web::Error>(bytes),
        Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
    });

    // 6. Envoi de la réponse HTTP avec les headers SSE obligatoires
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}
