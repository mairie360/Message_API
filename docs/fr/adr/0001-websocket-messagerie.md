# ADR 0001 : passer le temps réel de la messagerie du SSE au WebSocket

- Statut : **proposée** (MAIR-656), à valider par l'équipe lors de la revue des trois PR de MAIR-656.
- Date : 2026-11-20
- Tickets : epic MAIR-655 ; MAIR-656 (cette décision et le contrat), MAIR-657 (Message_API), MAIR-658
  (BFF_Message), MAIR-818 (Messages_Web_Service), MAIR-660 (Deploiment), MAIR-661 (ZAP, k6, newman),
  MAIR-662 (retrait du SSE)
- Contrat : [`../temps-reel-websocket.md`](../temps-reel-websocket.md)

## Contexte

Le service de messagerie pousse ses notifications par **Server-Sent Events** :

```
navigateur ──flux fetch──► Messages_Web_Service  /stream  (src/lib/message-stream-proxy.ts, cookie → Bearer)
                             ▼
                           BFF_Message           GET /stream          (chat_id → conversation-<id>)
                             ▼
                           Message_API           GET /api/v1/stream   (sse::state::AppState, relais Redis)
```

- `Message_API` garde les flux ouverts dans `sse::state::AppState` (5 au plus par utilisateur, le plus ancien est
  évincé), diffuse les `ChatEvent` de tous les réplicas par le relais Redis (`message-api:sse:chat-events`),
  revérifie le jeton toutes les 30 s et ferme à l'expiration ou à la révocation. Les signaux ne portent aucun
  contenu : `NEW_MSG`, `MSG_EDITED`, `MSG_DELETED` aux membres sauf l'auteur, `CHAT_READ` (avec `unread_count`)
  aux seuls flux du lecteur. Les clients relisent le changement par `GET /api/v1/{chat_id}/?since=` (MAIR-524).
- `BFF_Message` relaie le flux, traduit `chat_id` en `conversation-<id>`, abandonne `message_id` et
  `unread_count`, ferme après `MESSAGE_STREAM_MAX_MS` (5 min) et signale un refus amont par un
  `event: stream_error`, car l'en-tête de la réponse est déjà parti (MAIR-561).
- `Messages_Web_Service` lit le flux avec `fetch` + `ReadableStream` (pas `EventSource`) et ne recharge toutes les
  2 s (`MESSAGE_REFRESH_INTERVAL_MS`) que tant que le flux n'est pas connecté. L'epic MAIR-655 et le
  `CONTRACT.md` du BFF le décrivent autrement (aucun consommateur / un `EventSource`) : les deux sont périmés.

Le SSE ne va que du serveur vers le client, tient une réponse HTTP par client sur trois sauts et ne peut plus
signaler un refus une fois l'en-tête envoyé. L'epic vise un plus haut débit et davantage de connexions
simultanées ; ce gain **n'est pas** acquis par le seul changement de protocole et sera mesuré par le scénario k6
de MAIR-661 face au scénario SSE.

Contraintes relevées dans le code et le chart :

1. **Seuls les fronts sont exposés.** L'Ingress de `mairie360-stack` route `<front>.<domaine>` vers les fronts ;
   BFF et API sont internes au cluster.
2. **Le BFF ne lit aucun cookie.** Le proxy du front (`src/lib/bff-proxy.ts`, `proxyPublishedBffRequest` de
   `@mairie360/lib-components/next`) vérifie l'origine, transforme le cookie HttpOnly `accessToken` en
   `Authorization: Bearer` et renouvelle la session.
3. **Un route handler Next.js ne peut pas accepter un `Upgrade` WebSocket.** Le front exécute le `server.js`
   autonome généré par `next build` (`output: "standalone"`).
4. Un `WebSocket` navigateur ne peut pas poser d'en-tête `Authorization` et **ne voit pas le statut HTTP** d'une
   poignée de main refusée (seulement un `error` puis un `close` de code 1006).

## Options

### Transport

| Option | Pour | Contre |
|---|---|---|
| **Garder le SSE** | Fonctionne aujourd'hui de bout en bout, avec repli sur le polling. | Un seul sens ; un refus après l'en-tête devient un `stream_error` dans le flux ; une longue réponse HTTP par saut. |
| **WebSocket** | Bidirectionnel (ping, codes de fermeture explicites), une connexion TCP promue par saut, un refus de poignée de main est un vrai statut HTTP entre serveurs. | Du code neuf sur trois couches, ingress à vérifier (MAIR-660), le navigateur ne lit pas les statuts de poignée de main. |

### Chemin du navigateur jusqu'au BFF

| Option | Pour | Contre |
|---|---|---|
| **A. Serveur Node du front** (retenue) | Le BFF reste privé ; le front garde la maîtrise du cookie et de la vérification d'origine, comme pour toutes les autres routes ; aucune route d'ingress vers un BFF. | Le front quitte le `server.js` généré pour un petit serveur personnalisé (Dockerfile et commande de démarrage modifiés). |
| B. Route d'ingress directe vers le BFF | Serveur du front inchangé. | BFF exposé publiquement ; le BFF doit lire le cookie et vérifier `Origin` (exception à la règle 2) ; NetworkPolicy ouverte de l'ingress vers un BFF. |
| C. Ticket à usage unique | Aucun cookie lu par le BFF. | BFF exposé comme en B ; un ticket à usage unique demande un stockage partagé que le BFF n'a pas. |

### Transition

| Option | Pour | Contre |
|---|---|---|
| Bascule directe | Moins de code vivant en même temps. | Pas de retour arrière pendant un déploiement progressif où des réplicas des deux versions coexistent. |
| **Coexistence puis retrait** (retenue) | Le SSE reste le repli jusqu'à la validation du WebSocket en dev et en staging ; retour arrière = remettre le front sur le SSE. | Deux transports maintenus pendant un sprint. |

## Décision

1. **WebSocket de bout en bout**, avec le contrat d'événements du SSE inchangé : les mêmes quatre types de signal,
   sans contenu, les mêmes destinataires, le rattrapage par lectures `?since=`. Voir
   [`../temps-reel-websocket.md`](../temps-reel-websocket.md).
2. **Option A** : `Messages_Web_Service` exécute un petit serveur Node autour de Next qui traite lui-même
   l'`Upgrade` de `/ws` (vérification d'origine, cookie → Bearer, cookies retirés) et le relaie vers
   `BFF_Message` `GET /ws`, qui ouvre `Message_API` `GET /api/v1/ws` avec le même Bearer. Aucun jeton ne passe
   dans une URL.
3. **Coexistence** : `GET /api/v1/stream` et `GET /stream` du BFF restent intacts et fonctionnels jusqu'à la
   validation du WebSocket en dev puis en staging et la bascule du front ; MAIR-662 les retire ensuite (`feat!`).
4. **Un seul registre de connexions pour les deux transports** dans `Message_API` : la limite de 5 connexions par
   utilisateur compte ensemble SSE et WebSocket, et le même relais Redis et le même `event_manager` alimentent les
   deux.
5. L'objectif de capacité est mesuré, pas supposé : le scénario k6 WebSocket de MAIR-661 relève les connexions
   simultanées et le temps de livraison des trames à côté du scénario SSE ; les cibles sont fixées à l'usage réel.

## Conséquences

- `Message_API` ajoute `actix-ws` et un émetteur indépendant du transport dans `sse::state` (le SSE formate
  `data: …\n\n`, le WebSocket envoie une trame texte) ; le superviseur (revalidation à 30 s, minuterie
  d'expiration) est partagé. La route SSE continue de fonctionner sans changement (MAIR-657).
- `BFF_Message` ajoute un serveur et un client WebSocket (`ws`) sur son serveur HTTP Express ; `GET /ws` est
  déclarée dans son contrat OpenAPI (statuts 101, 401, 429, 503) avec le schéma de trame `MessageRealtimeEvent`,
  pour que le proxy du front l'autorise et que les tests de contrat la couvrent (MAIR-658).
- `Messages_Web_Service` remplace `node server.js` par son propre serveur, et son client remplace le lecteur
  SSE : WebSocket d'abord, repli SSE/polling conservé jusqu'à MAIR-662 (MAIR-818).
- L'Ingress n'a pas besoin de nouvelle route (le WebSocket passe par l'hôte du front), mais le middleware
  `buffering` de Traefik et les délais doivent laisser passer une connexion promue de longue durée (MAIR-660).
- newman ne sait pas ouvrir de WebSocket : les TI de MAIR-657/658/661 lancent un petit script Node `ws` depuis la
  stack newman.
