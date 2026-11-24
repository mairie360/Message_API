# Messaging real-time contract (WebSocket)

Reference contract of the messaging WebSocket for `Message_API`, `BFF_Message` and `Messages_Web_Service`
(MAIR-656, decision in [`adr/0001-messaging-websocket.md`](adr/0001-messaging-websocket.md)). OpenAPI cannot
describe frames: each layer declares its handshake route in its OpenAPI contract and points here for the frames.
Any change to this file is a contract change for the three repos.

## Endpoints

| Hop | Route | Caller authentication |
|---|---|---|
| browser → front | `GET /ws` on the front origin (`wss://messages.<domain>/ws`) | HttpOnly `accessToken` cookie (sent by the browser) and an `Origin` equal to the front's public origin |
| front → BFF | `GET /ws` on `BFF_Message` | `Authorization: Bearer <accessToken>`; no cookie forwarded |
| BFF → API | `GET /api/v1/ws` on `Message_API` | `Authorization: Bearer <same token>` (`asCaller`) |

- Every hop is a plain HTTP/1.1 `GET` with `Upgrade: websocket`. No subprotocol, no extension is required
  (`permessage-deflate` is not negotiated: frames are tiny).
- **No token, ticket or identifier in the URL** (URLs end up in logs). The query string is ignored.
- Each hop authenticates **before** upgrading and before opening the next hop, and answers a refusal with a real
  HTTP status and the usual error body of that layer:

| Status | API `/api/v1/ws` | BFF `/ws` | Front `/ws` |
|---|---|---|---|
| `101` | upgraded | upgraded, once the API accepted its own handshake | upgraded, once the BFF accepted |
| `400` | not a WebSocket handshake | not a WebSocket handshake | not a WebSocket handshake |
| `401` | missing, invalid or expired JWT, revoked session | missing or invalid session (before any upstream call); the API answered `401` or `404` | missing or expired cookie |
| `404` | the token is valid but its account is unknown or archived (as on every route of the API) | — | — |
| `403` | — | — | `Origin` missing or not the front's own |
| `429` | rate limit of `/api` | rate limit of the caller | — |
| `502` | — | the API refused otherwise or did not answer the handshake within 10 s | the BFF did not answer within 10 s |
| `503` | — | `MESSAGE_API_URL` not configured | BFF URL not configured, or `/ws` absent from the front's `contracts/openapi.json` |

A browser cannot read these statuses (it only sees `close` code `1006`): see *Client behaviour*.

## Frames server → client

UTF-8 **text** frames, one JSON object each. A client **ignores** a frame it cannot parse or whose `type` it does
not know (new types may be added without a contract version bump). Fields are only ever added.

Between the API and the BFF the frame is the API's `ChatSignal` ; the BFF
translates it to the front's identifiers:

| `type` | When | API frame (`ChatSignal`) | BFF/front frame (`MessageRealtimeEvent`) | Delivered to |
|---|---|---|---|---|
| `NEW_MSG` | a message was posted | `chat_id`, `message_id` | `conversationId`, `messageId` | every connection of the non-excluded members **but the author** |
| `MSG_EDITED` | a message was edited | `chat_id`, `message_id` | `conversationId`, `messageId` | the members but the editor |
| `MSG_DELETED` | a message was deleted (tombstone) | `chat_id`, `message_id` | `conversationId`, `messageId` | the members but the deleting user |
| `CHAT_READ` | the user acknowledged the chat (`POST /api/v1/{chat_id}/read/`) | `chat_id`, `message_id`, `unread_count` | `conversationId`, `messageId`, `unreadCount` | **every connection of that user only** (other tabs and devices) |

Examples:

```json
{"type":"NEW_MSG","chat_id":5,"message_id":118}
{"type":"CHAT_READ","chat_id":5,"message_id":118,"unread_count":0}
```

```json
{"type":"NEW_MSG","conversationId":"conversation-5","messageId":"message-118"}
{"type":"CHAT_READ","conversationId":"conversation-5","messageId":"message-118","unreadCount":0}
```

- `message_id` is absent only from a signal relayed by an API replica older than MAIR-524 (then always
  `NEW_MSG`); the BFF then omits `messageId`.
- The BFF forwards **only** the four types above and drops any other or invalid upstream frame. It never widens
  nor narrows the recipients the API chose.
- A signal says *that* something changed, never *what*: on `NEW_MSG`, `MSG_EDITED`, `MSG_DELETED` the client reads
  the conversation since its cursor (`GET /conversations/{id}` → API `GET /api/v1/{chat_id}/?since=&since_id=`) and
  refreshes the list line (`?since=`); on `CHAT_READ` it sets the unread badge to `unreadCount`.
- Delivery is **at most once per connection and not guaranteed**: a signal may be duplicated (Redis relay
  reconnecting) or lost during a reconnection (a full buffer closes the socket with `1013` rather than drop silently). Reads since a cursor are idempotent, so a duplicate costs one
  read and a loss is caught up at the next (re)connection.

## Frames client → server

None. The channel is server → client for application data. A **data** frame (text or binary) sent by a client is
never forwarded: the BFF drops it, and the API closes a connection that sends one with `1003`. The front relay
passes frames through unchanged, so the BFF's rule applies to the browser. Maximum accepted frame size at every
hop: **4 KiB** (`1009` beyond). Control frames (ping, pong, close) are handled by the
WebSocket libraries.

## Liveness

- The **API** sends a WebSocket `ping` every **15 s** to each connection and closes it (`1001`) when no `pong` came
  back for **45 s**; the connection is then removed from the registry. The BFF and the browser answer pings
  automatically.
- The **BFF** pings its client the same way (15 s, closed after 45 s without pong), independently of the API.
- The front relay is transparent: it forwards ping/pong frames.
- The browser cannot see pings: it relies on the close events, on `online` / `visibilitychange` (see below) and on
  the BFF's maximum lifetime.

## Server-side closure

| Code | Sent by | Meaning | Client action |
|---|---|---|---|
| `1000` | any | normal closure (the client left) | — |
| `1001` | API, BFF, front | going away: server shutdown, pong timeout, BFF maximum lifetime reached | reconnect with backoff |
| `1003` | API | the client sent a data frame | do not send data; reconnect with backoff |
| `1009` | any | frame larger than 4 KiB | reconnect with backoff |
| `1011` | any | unexpected server error | reconnect with backoff |
| `1013` | API, BFF, front | API: the send buffer of this connection overflowed (a signal would be lost); BFF, front: the upstream hop closed abnormally or is unreachable | reconnect with backoff (the catch-up on `open` reads what was missed) |
| `4001` | API (relayed) | the session expired, was revoked, or the account was removed (token checked every **30 s** and at its `exp`) | **do not reconnect** before a session renewal (see below) |
| `4008` | API (relayed) | evicted: the same user opened a 6th connection on this API replica, the oldest is closed | **do not reconnect automatically** (it would evict another tab); reconnect on focus or explicit retry |

- The **BFF** propagates a close in both directions: a client close closes the upstream with the same code when it
  is a valid code to send (`1000`, `1001`, `4xxx`), `1000` otherwise; an upstream close with `4001` / `4008` /
  `1001` is relayed with the same code and reason, any abnormal upstream end (`1006`, error) becomes `1013`.
- The **BFF** closes with `1001` after `MESSAGE_WS_MAX_MS` (default **30 min**, ± 10 % jitter so clients do not
  reconnect at once), which rebalances connections across replicas.
- Close reasons are short English strings for logs (`session expired`, `evicted`, `upstream unavailable`…); clients
  act on the code only.

## Limits

- **5 connections per user per API replica**: the 6th closes the oldest with `4008`. With several replicas a user may hold up to 5 on
  each; the Redis relay delivers to all of them.
- The handshake goes through the usual per-user rate limit of each layer (the API's `/api` limiter, the BFF's
  caller limiter); the open connection does not consume the limit.
- Per-connection send buffer of the API: **10 frames**. When it is full, the WebSocket is closed with `1013` instead
  of losing the signal silently: the client reconnects and catches up since its cursors on `open`.

## Client behaviour (Messages_Web_Service)

1. Open `/ws` on the front origin (`wss://` for an HTTPS page, `ws://` for a plain HTTP one, e.g. a local run) once the session is known (bootstrap done) and the page is visible.
2. On `open`: read the conversation list and the open conversation **since their cursors** (catch-up for anything
   missed while disconnected), then stop the 2 s polling.
3. On `close` (or `error` then `close`):
   - `4001`, or `1006` before the first `open` of this attempt (a refused handshake looks the same): check the
     session with one REST read through the proxy (it renews the session when it can). Refused (`401`) → go to
     Login. Never loop on `4001`. Valid or renewed → count the attempt as a failure and continue as for any other
     code below, so repeated `1006` with a valid session (BFF or ingress down) reach the 5-failure fallback; the
     session check runs at most once per sequence of failures.
   - `4008`: stay disconnected, poll every 2 s, show the explicit retry; reconnect on focus or retry.
   - anything else: reconnect after `min(30 s, 1 s × 2^attempt)` ± 20 % jitter. After **5** consecutive failures
     without an `open`, fall back to the 2 s polling and show a French message without blocking the page
     (« La réception en temps réel est indisponible. Les messages sont actualisés automatiquement. ») with a retry
     action; keep trying on `online`, focus and retry.
4. On `visibilitychange` to hidden, close with `1000` after 60 s hidden; when visible again, reconnect and catch
   up as in step 2.
5. Do not deduplicate signals: two `MSG_EDITED` of the same message are two edits. Each signal triggers a read since
   the cursor, which is idempotent, so a duplicate only costs one read; reads of the same conversation may be
   coalesced while one is in flight, as long as one more read follows the last signal.
6. A browser without the `WebSocket` API, like a failing WebSocket, uses the 2 s polling (step 3).

## SSE removal

The WebSocket replaced the SSE stream: `GET /api/v1/stream`, BFF `GET /stream`, their OpenAPI
operations and `MessageStreamEvent` are removed (`feat!` / `BREAKING CHANGE:` in the API and the BFF) and answer `404`.

## Tests expected per layer

| Layer | TU | TI technical | TI business |
|---|---|---|---|
| API (MAIR-657) | registry (eviction across transports, removal), close on expiry/revocation, frame serialization | handshake `401` without/with invalid token; frame after a `POST` of message | A and B members, C outsider: only B gets `NEW_MSG`; `CHAT_READ` to A's other connection only; 6th connection evicts with `4008` |
| BFF (MAIR-658) | translation, unknown frames dropped, close-code mapping, max lifetime | handshake `401` before any upstream call, `503` unconfigured; frame relayed after a `POST` | same journey through the BFF ids |
| Front (MAIR-818) | reconnection policy, close-code handling, fallback | server upgrade: `403` bad origin, `401` no cookie, cookie stripped and Bearer set | message appears without polling; reconnection catches up without duplicate |
| Stacks (MAIR-661) | — | ZAP rules for `/ws`, k6 WebSocket scenario | newman + a Node `ws` script for the journeys |

### Test stacks

- **newman** (`integration_test.sh`) checks the refused handshakes (`401` without or with a forged JWT, with the
  plain-text error body; `400` without `Upgrade`). newman cannot open a WebSocket, so the journey over real sockets
  is `tests/websocket/journey.mjs` (plain Node), run on the same stack once the collection passed: A and B members,
  C outsider; A posts: only B gets `NEW_MSG`, once; B's 6th connection evicts the oldest with `4008` and the 5 others
  keep receiving.
- **ZAP** (`security_test.sh`): `/api/v1/ws` is scanned like any route. ZAP cannot upgrade, so it sees the plain
  `GET` (`401` without a token, `400` without `Upgrade`, plain text, no alert). Only the Swagger UI stays out of the scope of rule
  `100001`.
- **k6** (`performance_test.sh`): the `websocket` scenario holds one socket per VU (20 in the `ci` profile, 200 in
  `stress`) while the Admin posts in each VU's chat every 2 s. Every post must come back as a `NEW_MSG` frame
  (`ws_frames_lost == 0`), `ws_frame_delivery` (start of the `POST` → frame) has a 1 s `p(95)` and the handshake
  (`ws_connecting`) a 500 ms `p(95)`, on top of the strict `checks`, `http_req_failed` and `dropped_iterations`. The
  `stream` scenario only counts the plain `GET /api/v1/ws` (`400`) for the coverage.
