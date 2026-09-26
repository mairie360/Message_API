// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// Three scenarios share the spec:
// - `reads`: the GET operations under the historical profile (ramp up to 20 VUs), against the
//   fixtures created once in setup() and removed in teardown();
// - `writes`: every other operation with 2 VUs. Each handler is self-contained: it creates what it
//   needs through `fixture()`, sends its request, then deletes what it created, so the handlers
//   do not depend on their order and the database ends as it started;
// - `stream`: GET /api/v1/stream alone, 1 VU every few seconds. The SSE response never ends, so
//   every call ends on a k6 timeout, which k6 logs as a warning: keeping it out of the 20 VUs
//   keeps the CI log readable.
import http from 'k6/http';
import { check, fail, sleep } from 'k6';
import { createCoverage, loadSpec } from '/coverage.js';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:3003').replace(/\/+$/, '');

// Static HS256 JWT (sub=1, the Admin seeded by liquibase, role=admin, exp=2100, signed with the
// stack's JWT_SECRET=b"secret"), the same one ZAP injects. Administrators bypass the chat
// membership checks, so every operation answers for any chat.
const TOKEN =
  __ENV.JWT ||
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjo0MTAyNDQ0ODAwfQ.xCeBe_2QxRlXW8WXr3t6F69wbEHA93HbP_7l4OTJwjA';
const AUTH = { Authorization: `Bearer ${TOKEN}` };

// Plain `User` accounts seeded by init-test.sql.
const MEMBER_ID = 2;
const OTHER_MEMBER_ID = 3;

// p(95) latency budget of each family of operations, in ms (reference machine).
const READ_BUDGET_MS = 200;
const WRITE_BUDGET_MS = 500;
// GET /stream never ends by itself (SSE): the request is held this long, then cut by k6.
const STREAM_HOLD = '1s';
const STREAM_BUDGET_MS = 1500;

const READ_METHODS = ['get', 'head', 'options'];
const STREAM_PATH = '/api/v1/stream';

/** The served spec restricted to the operations whose method and path pass `keep`. */
function specSubset(spec, keep) {
  const paths = {};
  for (const path of Object.keys(spec.paths)) {
    const kept = {};
    for (const method of Object.keys(spec.paths[path])) {
      if (keep(method, path)) kept[method] = spec.paths[path][method];
    }
    if (Object.keys(kept).length > 0) paths[path] = kept;
  }
  return Object.assign({}, spec, { paths });
}

/**
 * Raw call for the fixtures of setup(), teardown() and the write handlers, outside the coverage
 * count. Tagged `op: fixture` so it stays out of the per-operation latency thresholds, but it
 * still counts in `http_req_failed`. Aborts the handler (or setup) on a non-2xx answer.
 */
function fixture(method, path, body) {
  const res = http.request(
    method,
    `${BASE_URL}${path}`,
    body === undefined ? null : JSON.stringify(body),
    { headers: Object.assign({ 'Content-Type': 'application/json' }, AUTH), tags: { op: 'fixture' } },
  );
  if (res.status < 200 || res.status >= 300) {
    fail(`fixture ${method} ${path} answered ${res.status}: ${res.body}`);
  }
  return res;
}

/** Chat of the Admin (added automatically, never listed in `members`) and `memberIds`. */
function createChat(name, memberIds) {
  return fixture('POST', '/api/v1/', { name, members: memberIds }).json('id');
}

function deleteChat(chatId) {
  fixture('DELETE', `/api/v1/${chatId}/`);
}

function postMessage(chatId, content) {
  return fixture('POST', `/api/v1/${chatId}/messages/`, { content, sitation: null }).json('id');
}

function deleteMessage(chatId, messageId) {
  fixture('DELETE', `/api/v1/${chatId}/messages/${messageId}/`);
}

const spec = loadSpec();

const readHandlers = {
  'GET /health': ({ request }) => check(request(), { 'health 200': (r) => r.status === 200 }),
  'GET /api/v1/': ({ request }) => check(request(), { 'list chats 200': (r) => r.status === 200 }),
  'GET /api/v1/{chat_id}/': ({ request, data }) =>
    check(request({ path: { chat_id: data.chatId } }), { 'get chat 200': (r) => r.status === 200 }),
  'GET /api/v1/{chat_id}/users/': ({ request, data }) =>
    check(request({ path: { chat_id: data.chatId } }), {
      'list chat users 200': (r) => r.status === 200,
    }),
};

const streamHandlers = {
  // SSE: the response never completes, so k6 cuts it after STREAM_HOLD (error 1050, status 0).
  // That timeout is the expected outcome (the stream stayed open), hence the responseCallback
  // keeping it out of http_req_failed; a refused stream (401, 500) answers at once and fails.
  'GET /api/v1/stream': ({ request }) =>
    check(
      request({ params: { timeout: STREAM_HOLD, responseCallback: http.expectedStatuses(0, 200) } }),
      { 'stream stays open': (r) => r.error_code === 1050 || r.status === 200 },
    ),
};

const writeHandlers = {
  'POST /': ({ request }) => check(request(), { 'hello 200': (r) => r.status === 200 }),

  // Chats: create → delete.
  'POST /api/v1/': ({ request }) => {
    const res = request({ body: { name: 'k6 create chat', members: [MEMBER_ID] } });
    check(res, { 'create chat 200': (r) => r.status === 200 });
    if (res.status === 200) deleteChat(res.json('id'));
  },
  'DELETE /api/v1/{chat_id}/': ({ request }) => {
    const chatId = createChat('k6 delete chat', [MEMBER_ID]);
    check(request({ path: { chat_id: chatId } }), { 'delete chat 204': (r) => r.status === 204 });
  },

  // Messages of the write sandbox chat: post → patch → delete.
  'POST /api/v1/{chat_id}/messages/': ({ request, data }) => {
    const res = request({
      path: { chat_id: data.writeChatId },
      body: { content: 'La réunion est décalée à 15h.', sitation: null },
    });
    check(res, { 'post message 200': (r) => r.status === 200 });
    if (res.status === 200) deleteMessage(data.writeChatId, res.json('id'));
  },
  'PATCH /api/v1/{chat_id}/messages/{message_id}/': ({ request, data }) => {
    const messageId = postMessage(data.writeChatId, 'k6 message to edit');
    check(
      request({
        path: { chat_id: data.writeChatId, message_id: messageId },
        body: { content: 'La réunion est finalement décalée à 16h.' },
      }),
      { 'patch message 200': (r) => r.status === 200 },
    );
    deleteMessage(data.writeChatId, messageId);
  },
  'DELETE /api/v1/{chat_id}/messages/{message_id}/': ({ request, data }) => {
    const messageId = postMessage(data.writeChatId, 'k6 message to delete');
    check(request({ path: { chat_id: data.writeChatId, message_id: messageId } }), {
      'delete message 204': (r) => r.status === 204,
    });
  },

  // Members, on a chat of their own: two VUs adding the same user to a shared chat would
  // answer 409.
  'POST /api/v1/{chat_id}/users/': ({ request }) => {
    const chatId = createChat('k6 add member', [MEMBER_ID]);
    check(request({ path: { chat_id: chatId }, body: { users_id: [OTHER_MEMBER_ID] } }), {
      'add chat users 200': (r) => r.status === 200,
    });
    deleteChat(chatId);
  },
  'DELETE /api/v1/{chat_id}/users/{user_id}/': ({ request }) => {
    const chatId = createChat('k6 remove member', [MEMBER_ID, OTHER_MEMBER_ID]);
    check(request({ path: { chat_id: chatId, user_id: OTHER_MEMBER_ID } }), {
      'remove chat user 204': (r) => r.status === 204,
    });
    deleteChat(chatId);
  },
};

const reads = createCoverage(readHandlers, {
  spec: specSubset(spec, (method, path) => READ_METHODS.includes(method) && path !== STREAM_PATH),
});
const stream = createCoverage(streamHandlers, {
  spec: specSubset(spec, (method, path) => READ_METHODS.includes(method) && path === STREAM_PATH),
});
const writes = createCoverage(writeHandlers, {
  spec: specSubset(spec, (method) => !READ_METHODS.includes(method)),
});

/** One `p(95)` threshold per operation (`op` tag) of `coverage`. */
function latencyThresholds(coverage, budgetMs) {
  const thresholds = {};
  for (const operation of coverage.operations) {
    thresholds[`http_req_duration{op:${operation.op}}`] = [`p(95)<${budgetMs}`];
  }
  return thresholds;
}

export const options = {
  scenarios: {
    reads: {
      executor: 'ramping-vus',
      exec: 'readScenario',
      stages: [
        { duration: '30s', target: 20 }, // Ramp up to 20 virtual users
        { duration: '1m', target: 20 }, // Hold
        { duration: '10s', target: 0 }, // Ramp down
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: 2,
      duration: '1m40s',
    },
    stream: {
      executor: 'constant-vus',
      exec: 'streamScenario',
      vus: 1,
      duration: '1m40s',
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(stream, STREAM_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    http_req_failed: ['rate<0.01'], // Less than 1% errors
  },
};

/** Read fixture: a chat with two members and a message; plus the sandbox of the writes. */
export function setup() {
  const chatId = createChat('k6 read fixture', [MEMBER_ID]);
  postMessage(chatId, 'k6 fixture');
  const writeChatId = createChat('k6 write sandbox', [MEMBER_ID]);
  return { chatId, writeChatId };
}

export function teardown(data) {
  deleteChat(data.chatId);
  deleteChat(data.writeChatId);
}

export function readScenario(data) {
  reads.run({ headers: AUTH, data });
  sleep(1);
}

export function writeScenario(data) {
  writes.run({ headers: AUTH, data });
  sleep(1);
}

export function streamScenario(data) {
  stream.run({ headers: AUTH, data });
  sleep(4);
}
