use std::time::Duration;

use crate::sse::state::{AppState, ChatSignal};
use actix_web::{get, web, HttpResponse, Responder};
use mairie360_api_lib::security::AuthenticatedUser;
use tokio::sync::mpsc;
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
                   Un seul flux est retenu par utilisateur : rouvrir ce endpoint remplace le \
                   précédent, qui cesse alors d'être alimenté. Le `body` documenté ci-dessous \
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

    // 2. Création d'un canal mpsc aligné avec le type de votre AppState
    let (tx, rx) = mpsc::channel::<Result<actix_web::web::Bytes, String>>(10);

    // 3. Enregistrement du Sender (le tuyau) dans la DashMap de l'état global
    state.online_agents.insert(user_id, tx.clone());

    // 4. Lancement d'un ping (keep-alive) en tâche de fond toutes les 15 secondes
    let state_clone = state.clone();
    let user_id_clone = user_id;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));

        // Le premier tick d'interval arrive immédiatement, on le consomme
        interval.tick().await;

        loop {
            interval.tick().await;

            let ping_bytes = actix_web::web::Bytes::from(": ping\n\n");

            // On envoie Ok(ping_bytes) car le canal attend un Result
            if tx.try_send(Ok(ping_bytes)).is_err() {
                state_clone.online_agents.remove(&user_id_clone);
                break;
            }
        }
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
