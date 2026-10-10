// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// High load on a volume seed (MAIR-474): the performance stack also runs init-perf.sql (2 000
// agents in 16 group chats each, 1 000 direct chats, 25 messages per chat and 10 000 in the hot
// chat 100000). Four scenarios share the spec:
// - `reads`: the GET operations, ramping up to the read VUs of the profile (PROFILES), as a random seeded agent (token signed
//   here with the run's JWT_SECRET): their chat list at a random offset, one of their chats (the
//   hot one at a random `before`, a third of the time) and its members;
// - `writes`: every other operation with the write VUs of the profile. Each handler is self-contained: it creates what it
//   needs through `fixture()`, sends its request, then deletes what it created, so the handlers
//   do not depend on their order and the database ends as it started;
// - `stream`: GET /api/v1/stream alone, 1 VU every few seconds. The SSE response never ends, so
//   every call ends on a k6 timeout, which k6 logs as a warning: keeping it out of the 20 VUs
//   keeps the CI log readable;
// - `chats_rush`: `GET /api/v1/` as agents at the fixed arrival rate of the profile, failing if k6 has to drop
//   iterations (the API no longer keeps up).
import http from 'k6/http';
import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import { check, fail, sleep } from 'k6';
import { createCoverage, loadSpec } from '/coverage.js';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:3003').replace(/\/+$/, '');

// HS256 JWT of the Admin seeded by liquibase (sub=1, role=admin), forged by performance_test.sh
// with the random JWT_SECRET of the run (MAIR-428): no token is committed. Administrators bypass
// the chat membership checks, so every operation answers for any chat.
const TOKEN = __ENV.JWT;
if (!TOKEN) {
  throw new Error('JWT is not set: run ./performance_test.sh, which forges it');
}
const AUTH = { Authorization: `Bearer ${TOKEN}` };

// Secret of the run, to sign the tokens of the seeded agents.
const JWT_SECRET = __ENV.JWT_SECRET;
if (!JWT_SECRET) {
  throw new Error('JWT_SECRET is not set: run ./performance_test.sh, which generates it');
}

// Rows of init-perf.sql: agent 400001 + r is a member of chat 100000 + r (and 102000 + r).
const AGENTS = { first: 400001, count: 2000 };
const GROUP_CHATS = { first: 100000, count: 4000 };
const CHATS_PER_AGENT = 16;
const HOT_CHAT_ID = 100000;
const HOT_MEMBERS = 8; // agents 400001 + 250 * k
const HOT_MESSAGES = 10000;
const PAGE = 50;

// Load profile (MAIR-474), K6_PROFILE:
// - `ci` (default): what the CI runner holds with the same strict thresholds. The runner
//   (ubuntu-latest, 4 vCPU) hosts the API, Postgres, Redis and k6 together;
// - `stress`: the high load, run by hand (`K6_PROFILE=stress ./performance_test.sh`) to find
//   the breaking point on a larger machine, not on every push.
const PROFILES = {
  ci: { readVus: 30, writeVus: 4, rushRate: 30 },
  stress: { readVus: 100, writeVus: 10, rushRate: 100 },
};
const PROFILE = PROFILES[__ENV.K6_PROFILE || 'ci'];
if (!PROFILE) throw new Error(`Unknown K6_PROFILE ${__ENV.K6_PROFILE}: ${Object.keys(PROFILES).join(', ')}`);

// Fixed-rate `GET /api/v1/` as agents.
const CHATS_RUSH_RATE = PROFILE.rushRate; // requests per second
const CHATS_RUSH_BUDGET_MS = 200;

const randomInt = (max) => Math.floor(Math.random() * max);

const agentTokens = {};

/** `Authorization` header of seeded agent `id`, an HS256 token signed with the run's secret. */
function agentAuth(id) {
  if (!agentTokens[id]) {
    const part = (value) => encoding.b64encode(JSON.stringify(value), 'rawurl');
    const unsigned = `${part({ alg: 'HS256', typ: 'JWT' })}.${part({ sub: String(id), role: 'user', exp: Math.floor(Date.now() / 1000) + 7200 })}`;
    agentTokens[id] = { Authorization: `Bearer ${unsigned}.${crypto.hmac('sha256', JWT_SECRET, unsigned, 'base64rawurl')}` };
  }
  return agentTokens[id];
}

/** A random seeded agent and one of its group chats. */
function randomAgent() {
  const rank = randomInt(AGENTS.count);
  const chatId = GROUP_CHATS.first + rank + (randomInt(2) === 0 || rank + AGENTS.count >= GROUP_CHATS.count ? 0 : AGENTS.count);
  return { headers: agentAuth(AGENTS.first + rank), chatId };
}

/** A random seeded agent by rank (it is a member of chat `GROUP_CHATS.first + rank`). */
function randomAgentRank() {
  const rank = randomInt(AGENTS.count);
  return { rank, headers: agentAuth(AGENTS.first + rank) };
}

/** A member of the hot chat. */
const hotMember = () => agentAuth(AGENTS.first + 250 * randomInt(HOT_MEMBERS));

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
  return fixture('POST', `/api/v1/${chatId}/messages/`, { content, citation: null }).json('id');
}

function deleteMessage(chatId, messageId) {
  fixture('DELETE', `/api/v1/${chatId}/messages/${messageId}/`);
}

const spec = loadSpec();

const readHandlers = {
  'GET /health': ({ request }) => check(request(), { 'health 200': (r) => r.status === 200 }),
  'GET /ready': ({ request }) => check(request(), { 'ready 200': (r) => r.status === 200 }),
  // A third of the lists search the agent's chats by the name of a member it shares chat `100000 + rank`
  // with (MAIR-507): the full name "Agent Perf <id>", which must find that chat.
  'GET /api/v1/': ({ request }) => {
    if (randomInt(3) === 0) {
      const { rank, headers } = randomAgentRank();
      const coMember = AGENTS.first + ((rank + 250) % AGENTS.count);
      const res = request({ query: { search: `Agent Perf ${coMember}` }, headers });
      check(res, {
        'search chats 200': (r) => r.status === 200,
        'search chats finds the chat shared with the member': (r) =>
          r.status === 200 && r.json('chats').some((chat) => chat.id === GROUP_CHATS.first + rank),
      });
      return;
    }
    check(request({ query: { offset: randomInt(CHATS_PER_AGENT) }, headers: randomAgent().headers }), {
      'list chats 200': (r) => r.status === 200,
      // Every seeded agent is in 16 group chats: an empty page means the seed was not loaded.
      'list chats reads the seed': (r) => r.status === 200 && r.json('chats').length > 0,
      // The list carries what it displays: a name for every chat (a direct chat is named after its
      // contact) and a member count.
      'list chats carries names and member counts': (r) =>
        r.status === 200 && r.json('chats').every((chat) => chat.name.length > 0 && chat.member_count >= 2),
    });
  },
  // A third of the reads page through the hot chat at a random depth.
  'GET /api/v1/{chat_id}/': ({ request, data }) => {
    const hot = randomInt(3) === 0;
    const agent = hot ? undefined : randomAgent();
    const chatId = hot ? HOT_CHAT_ID : agent.chatId;
    const res = hot
      ? request({
          path: { chat_id: HOT_CHAT_ID },
          query: { before: data.hotNewestId + 1 - randomInt(HOT_MESSAGES), limit: PAGE },
          headers: hotMember(),
        })
      : request({ path: { chat_id: chatId }, headers: agent.headers });
    check(res, {
      'get chat 200': (r) => r.status === 200,
      'get chat reads the seeded messages': (r) => r.status === 200 && r.json('messages').length > 0,
      // The header of the chat comes with the page (MAIR-507).
      'get chat carries its header': (r) =>
        r.status === 200 && r.json('chat.id') === chatId && r.json('chat.name').length > 0 && r.json('chat.member_count') === 8,
    });
  },
  'GET /api/v1/{chat_id}/users/': ({ request }) => {
    const agent = randomAgent();
    check(request({ path: { chat_id: agent.chatId }, headers: agent.headers }), {
      'list chat users 200': (r) => r.status === 200,
      'list chat users reads the seeded members': (r) => r.status === 200 && r.json('users').length >= 8,
      // The members come with their names (MAIR-507).
      'list chat users carries the names': (r) =>
        r.status === 200 && r.json('users').every((user) => user.first_name === 'Agent' && user.last_name.startsWith('Perf ')),
    });
  },
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
  // Chats: create → delete.
  'POST /api/v1/': ({ request }) => {
    const res = request({ body: { name: 'k6 create chat', members: [MEMBER_ID] } });
    check(res, { 'create chat 200': (r) => r.status === 200 });
    if (res.status === 200) deleteChat(res.json('id'));
  },
  // The direct chat of the Admin and MEMBER_ID: the same chat for every call (one per pair), so
  // both VUs may open it at once; teardown() deletes it.
  'POST /api/v1/direct/': ({ request }) => {
    check(request({ body: { contact_id: MEMBER_ID } }), {
      'open direct chat 200': (r) => r.status === 200,
    });
  },
  'DELETE /api/v1/{chat_id}/': ({ request }) => {
    const chatId = createChat('k6 delete chat', [MEMBER_ID]);
    check(request({ path: { chat_id: chatId } }), { 'delete chat 204': (r) => r.status === 204 });
  },

  // Messages of the write sandbox chat: post → patch → delete.
  'POST /api/v1/{chat_id}/messages/': ({ request, data }) => {
    const res = request({
      path: { chat_id: data.writeChatId },
      body: { content: 'La réunion est décalée à 15h.', citation: null },
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

  // Read acknowledgement, on a chat of its own: the Admin posts a message and acknowledges it.
  'POST /api/v1/{chat_id}/read/': ({ request }) => {
    const chatId = createChat('k6 acknowledge read', [MEMBER_ID]);
    const messageId = postMessage(chatId, 'k6 message to acknowledge');
    check(
      request({ path: { chat_id: chatId }, body: { readUntilMessageId: messageId } }),
      {
        'acknowledge read 200': (r) => r.status === 200,
        'own message is never unread': (r) => r.json('unread_count') === 0,
      },
    );
    deleteChat(chatId);
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
        { duration: '30s', target: Math.ceil(PROFILE.readVus / 2) },
        { duration: '30s', target: PROFILE.readVus },
        { duration: '2m', target: PROFILE.readVus }, // Hold
        { duration: '20s', target: 0 },
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: PROFILE.writeVus,
      duration: '3m20s',
    },
    stream: {
      executor: 'constant-vus',
      exec: 'streamScenario',
      vus: 1,
      duration: '3m20s',
    },
    chats_rush: {
      executor: 'constant-arrival-rate',
      exec: 'chatsRushScenario',
      startTime: '1m', // once the reads are at full load
      rate: CHATS_RUSH_RATE,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(stream, STREAM_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    'http_req_duration{op:chats_rush}': [`p(95)<${CHATS_RUSH_BUDGET_MS}`],
    dropped_iterations: ['count==0'], // the rush kept its rate
    // Strict (MAIR-474): one wrong status or one missing seeded row fails the run.
    checks: ['rate==1'],
    http_req_failed: ['rate==0'],
  },
};

/** The newest message of the hot chat (its page cursor) and the sandbox of the writes. */
export function setup() {
  const page = fixture('GET', `/api/v1/${HOT_CHAT_ID}/?limit=1`).json();
  const hotNewestId = page.messages[page.messages.length - 1].id;
  const writeChatId = createChat('k6 write sandbox', [MEMBER_ID]);
  return { hotNewestId, writeChatId };
}

export function teardown(data) {
  deleteChat(data.writeChatId);
  deleteChat(fixture('POST', '/api/v1/direct/', { contact_id: MEMBER_ID }).json('id'));
}

export function readScenario(data) {
  reads.run({ headers: AUTH, data });
  sleep(1);
}

export function writeScenario(data) {
  writes.run({ headers: AUTH, data });
  sleep(1);
}

export function chatsRushScenario() {
  const res = http.get(`${BASE_URL}/api/v1/`, { headers: randomAgent().headers, tags: { op: 'chats_rush' } });
  check(res, {
    'chats rush 200': (r) => r.status === 200,
    'chats rush reads the seed': (r) => r.status === 200 && r.json('chats').length > 0,
    'chats rush carries names and member counts': (r) =>
      r.status === 200 && r.json('chats').every((chat) => chat.name.length > 0 && chat.member_count >= 2),
  });
}

export function streamScenario(data) {
  stream.run({ headers: AUTH, data });
  sleep(4);
}
