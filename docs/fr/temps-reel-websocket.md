# Contrat temps réel de la messagerie (WebSocket)

Contrat de référence du WebSocket de messagerie pour `Message_API`, `BFF_Message` et `Messages_Web_Service`
(MAIR-656, décision dans [`adr/0001-websocket-messagerie.md`](adr/0001-websocket-messagerie.md)). OpenAPI ne
décrit pas les trames : chaque couche déclare sa route de poignée de main dans son contrat OpenAPI et renvoie ici
pour les trames. Toute modification de ce fichier est un changement de contrat pour les trois dépôts. La version
anglaise ([`../en/realtime-websocket.md`](../en/realtime-websocket.md)) fait foi en cas d'écart.

## Points d'entrée

| Saut | Route | Authentification de l'appelant |
|---|---|---|
| navigateur → front | `GET /ws` sur l'origine du front (`wss://messages.<domaine>/ws`) | cookie HttpOnly `accessToken` (envoyé par le navigateur) et un `Origin` égal à l'origine publique du front |
| front → BFF | `GET /ws` sur `BFF_Message` | `Authorization: Bearer <accessToken>` ; aucun cookie transmis |
| BFF → API | `GET /api/v1/ws` sur `Message_API` | `Authorization: Bearer <même jeton>` (`asCaller`) |

- Chaque saut est un `GET` HTTP/1.1 classique avec `Upgrade: websocket`. Aucun sous-protocole ni extension n'est
  requis (`permessage-deflate` n'est pas négocié : les trames sont minuscules).
- **Aucun jeton, ticket ni identifiant dans l'URL** (les URL finissent dans les journaux). La query string est
  ignorée.
- Chaque saut authentifie **avant** de promouvoir la connexion et avant d'ouvrir le saut suivant, et répond à un
  refus par un vrai statut HTTP avec le corps d'erreur habituel de sa couche :

| Statut | API `/api/v1/ws` | BFF `/ws` | Front `/ws` |
|---|---|---|---|
| `101` | connexion promue | promue, une fois la poignée de main de l'API acceptée | promue, une fois celle du BFF acceptée |
| `400` | pas une poignée de main WebSocket | pas une poignée de main WebSocket | pas une poignée de main WebSocket |
| `401` | JWT absent, invalide ou expiré, session révoquée | session absente ou invalide (avant tout appel amont) ; l'API a répondu `401` ou `404` | cookie absent ou expiré |
| `404` | jeton valide mais compte inconnu ou archivé (comme sur toutes les routes de l'API) | — | — |
| `403` | — | — | `Origin` absent ou différent de celui du front |
| `429` | limite de débit de `/api` | limite de débit de l'appelant | — |
| `502` | — | l'API a refusé autrement ou n'a pas répondu à la poignée de main en 10 s | le BFF n'a pas répondu en 10 s |
| `503` | — | `MESSAGE_API_URL` non configurée | URL du BFF non configurée, ou `/ws` absent du `contracts/openapi.json` du front |

Un navigateur ne peut pas lire ces statuts (il ne voit que le code de fermeture `1006`) : voir *Comportement du
client*.

## Trames serveur → client

Trames **texte** UTF-8, un objet JSON chacune. Un client **ignore** une trame illisible ou dont il ne connaît pas
le `type` (de nouveaux types peuvent être ajoutés sans changer de version de contrat). On n'ajoute jamais que des
champs.

Entre l'API et le BFF, la trame est le `ChatSignal` de l'API (identique à la charge `data:` du SSE) ; le BFF la
traduit dans les identifiants du front :

| `type` | Quand | Trame API (`ChatSignal`) | Trame BFF/front (`MessageRealtimeEvent`) | Destinataires |
|---|---|---|---|---|
| `NEW_MSG` | un message a été posté | `chat_id`, `message_id` | `conversationId`, `messageId` | toutes les connexions des membres non exclus **sauf l'auteur** |
| `MSG_EDITED` | un message a été modifié | `chat_id`, `message_id` | `conversationId`, `messageId` | les membres sauf l'auteur de la modification |
| `MSG_DELETED` | un message a été supprimé (tombstone) | `chat_id`, `message_id` | `conversationId`, `messageId` | les membres sauf l'auteur de la suppression |
| `CHAT_READ` | l'utilisateur a acquitté la conversation (`POST /api/v1/{chat_id}/read/`) | `chat_id`, `message_id`, `unread_count` | `conversationId`, `messageId`, `unreadCount` | **toutes les connexions de cet utilisateur uniquement** (autres onglets et appareils) |

Exemples :

```json
{"type":"NEW_MSG","chat_id":5,"message_id":118}
{"type":"CHAT_READ","chat_id":5,"message_id":118,"unread_count":0}
```

```json
{"type":"NEW_MSG","conversationId":"conversation-5","messageId":"message-118"}
{"type":"CHAT_READ","conversationId":"conversation-5","messageId":"message-118","unreadCount":0}
```

- `message_id` n'est absent que d'un signal relayé par un réplica d'API antérieur à MAIR-524 (toujours `NEW_MSG`
  alors) ; le BFF omet alors `messageId`.
- Le BFF ne transmet **que** les quatre types ci-dessus et abandonne toute autre trame amont ou trame invalide. Il
  n'élargit ni ne restreint jamais les destinataires choisis par l'API.
- Un signal dit *qu'*un changement a eu lieu, jamais *lequel* : sur `NEW_MSG`, `MSG_EDITED`, `MSG_DELETED` le client
  relit la conversation depuis son curseur (`GET /conversations/{id}` → API
  `GET /api/v1/{chat_id}/?since=&since_id=`) et actualise la ligne de la liste (`?since=`) ; sur `CHAT_READ` il met
  le badge de non-lus à `unreadCount`.
- La livraison est **au plus une fois par connexion et non garantie** : un signal peut être dupliqué (reconnexion du
  relais Redis) ou perdu pendant une reconnexion (un tampon plein ferme le socket avec `1013` plutôt que de perdre en silence). Les lectures depuis un curseur sont idempotentes : un doublon
  coûte une lecture et une perte est rattrapée à la (re)connexion suivante.

## Trames client → serveur

Aucune. Le canal ne transporte des données que du serveur vers le client. Une trame de **données** (texte ou
binaire) envoyée par un client n'est jamais transmise : le BFF l'abandonne, et l'API ferme avec `1003` une connexion
qui en envoie. Le relais du front transmet les trames telles quelles, la règle du BFF s'applique donc au
navigateur. Taille maximale de trame acceptée à chaque saut : **4 Kio** (`1009` au-delà).

## Maintien de connexion

- L'**API** envoie un `ping` WebSocket toutes les **15 s** à chaque connexion et la ferme (`1001`) faute de `pong`
  pendant **45 s** ; la connexion est alors retirée du registre. Le BFF et le navigateur répondent automatiquement
  aux pings.
- Le **BFF** pingue son client de la même façon (15 s, fermeture après 45 s sans pong), indépendamment de l'API.
- Le relais du front est transparent : il transmet les trames ping/pong.
- Le navigateur ne voit pas les pings : il s'appuie sur les fermetures, sur `online` / `visibilitychange` (voir
  plus bas) et sur la durée de vie maximale imposée par le BFF.

## Fermeture par le serveur

| Code | Envoyé par | Signification | Action du client |
|---|---|---|---|
| `1000` | tous | fermeture normale (le client est parti) | — |
| `1001` | API, BFF, front | départ : arrêt du serveur, absence de pong, durée de vie maximale du BFF atteinte | se reconnecter avec temporisation |
| `1003` | API | le client a envoyé une trame de données | ne rien envoyer ; se reconnecter avec temporisation |
| `1009` | tous | trame de plus de 4 Kio | se reconnecter avec temporisation |
| `1011` | tous | erreur serveur inattendue | se reconnecter avec temporisation |
| `1013` | API, BFF, front | API : le tampon d'envoi de cette connexion a débordé (un signal serait perdu) ; BFF, front : le saut amont a fermé anormalement ou est injoignable | se reconnecter avec temporisation (le rattrapage à l'`open` relit ce qui a été manqué) |
| `4001` | API (relayé) | session expirée, révoquée ou compte supprimé (jeton revérifié toutes les **30 s** et à son `exp`) | **ne pas se reconnecter** avant un renouvellement de session (voir plus bas) |
| `4008` | API (relayé) | éviction : le même utilisateur a ouvert une 6e connexion sur ce réplica d'API, la plus ancienne est fermée | **ne pas se reconnecter automatiquement** (cela évincerait un autre onglet) ; se reconnecter au focus ou sur demande |

- Le **BFF** propage une fermeture dans les deux sens : une fermeture du client ferme l'amont avec le même code
  quand il est valide à l'envoi (`1000`, `1001`, `4xxx`), `1000` sinon ; une fermeture amont `4001` / `4008` /
  `1001` est relayée avec le même code et la même raison, toute fin anormale de l'amont (`1006`, erreur) devient
  `1013`.
- Le **BFF** ferme avec `1001` après `MESSAGE_WS_MAX_MS` (**30 min** par défaut, ± 10 % d'aléa pour que les clients
  ne se reconnectent pas tous en même temps), ce qui rééquilibre les connexions entre réplicas. Le
  `MESSAGE_STREAM_MAX_MS` du SSE est inchangé.
- Les raisons de fermeture sont de courtes chaînes anglaises pour les journaux (`session expired`, `evicted`,
  `upstream unavailable`…) ; les clients n'agissent que sur le code.

## Limites

- **5 connexions par utilisateur et par réplica d'API**, SSE et WebSocket comptés ensemble (un seul registre) : la
  6e ferme la plus ancienne avec `4008` (WebSocket) ou termine son flux (SSE). Avec plusieurs réplicas, un
  utilisateur peut en tenir 5 sur chacun ; le relais Redis les sert toutes.
- La poignée de main passe par la limite de débit habituelle de chaque couche (limiteur `/api` de l'API, limiteur
  par appelant du BFF) ; la connexion ouverte ne consomme pas la limite.
- Tampon d'envoi par connexion côté API : **10 trames**, comme pour le SSE. Plein, un WebSocket est fermé avec
  `1013` au lieu de perdre le signal en silence : le client se reconnecte et rattrape depuis ses curseurs à l'`open`.
  (Le SSE continue d'abandonner la trame pour cette seule connexion.)

## Comportement du client (Messages_Web_Service)

1. Ouvrir `wss://<origine du front>/ws` une fois la session connue (bootstrap terminé) et la page visible.
2. À l'`open` : relire la liste des conversations et la conversation ouverte **depuis leurs curseurs** (rattrapage de
   ce qui a été manqué hors connexion), puis arrêter le polling de 2 s.
3. Au `close` (ou `error` puis `close`) :
   - `4001`, ou `1006` avant le premier `open` de cette tentative (une poignée de main refusée y ressemble) :
     vérifier la session par une lecture REST via le proxy (qui renouvelle la session quand il le peut). Refusée
     (`401`) → aller vers Login. Jamais de boucle sur `4001`. Valide ou renouvelée → compter la tentative comme un
     échec et continuer comme pour tout autre code ci-dessous, afin que des `1006` répétés avec une session valide
     (BFF ou ingress indisponible) atteignent le repli après 5 échecs ; la session n'est vérifiée qu'une fois par
     suite d'échecs.
   - `4008` : rester déconnecté, interroger toutes les 2 s, afficher la reprise explicite ; se reconnecter au focus
     ou sur demande.
   - tout autre code : se reconnecter après `min(30 s, 1 s × 2^tentative)` ± 20 % d'aléa. Après **5** échecs
     consécutifs sans `open`, revenir au polling de 2 s et afficher un message en français sans bloquer la page
     (« La réception en temps réel est indisponible. Les messages sont actualisés automatiquement. ») avec une
     action de reprise ; continuer d'essayer sur `online`, au focus et à la reprise.
4. Quand la page est masquée depuis 60 s (`visibilitychange`), fermer avec `1000` ; redevenue visible, se
   reconnecter et rattraper comme à l'étape 2.
5. Ne pas dédupliquer les signaux : deux `MSG_EDITED` du même message sont deux modifications. Chaque signal
   déclenche une lecture depuis le curseur, idempotente, donc un doublon ne coûte qu'une lecture ; les lectures d'une
   même conversation peuvent être regroupées pendant qu'une est en cours, tant qu'une lecture suit le dernier signal.
6. Jusqu'à MAIR-662, le client SSE reste le repli quand `WebSocket` n'est pas disponible.

## Coexistence et retrait

Jusqu'à la validation du WebSocket en dev puis en staging (CA11 de MAIR-655), `GET /api/v1/stream` et
`GET /stream` du BFF restent disponibles et inchangés, et le front peut y revenir. MAIR-662 les retire, ainsi que
leurs opérations OpenAPI et `MessageStreamEvent`, avec un `feat!` / `BREAKING CHANGE:` dans l'API et le BFF.

## Tests attendus par couche

| Couche | TU | TI technique | TI métier |
|---|---|---|---|
| API (MAIR-657) | registre (éviction entre transports, retrait), fermeture à l'expiration/révocation, sérialisation des trames | poignée de main `401` sans jeton ou avec un jeton invalide ; trame après un `POST` de message | A et B membres, C extérieur : seul B reçoit `NEW_MSG` ; `CHAT_READ` à l'autre connexion de A seulement ; une 6e connexion évince avec `4008` |
| BFF (MAIR-658) | traduction, trames inconnues abandonnées, correspondance des codes de fermeture, durée de vie maximale | poignée de main `401` avant tout appel amont, `503` non configuré ; trame relayée après un `POST` | même parcours avec les identifiants du BFF |
| Front (MAIR-818) | politique de reconnexion, gestion des codes de fermeture, repli | upgrade du serveur : `403` mauvaise origine, `401` sans cookie, cookie retiré et Bearer posé | un message apparaît sans polling ; la reconnexion rattrape sans doublon |
| Stacks (MAIR-661) | — | règles ZAP pour `/ws`, scénario k6 WebSocket | newman + un script Node `ws` pour les parcours |
