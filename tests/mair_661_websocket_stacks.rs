//! The test stacks keep covering `GET /api/v1/ws` (MAIR-661): the integration stack runs the
//! WebSocket journey after newman, k6 holds real sockets under load with a frame delivery budget,
//! and ZAP scans `/api/v1/ws` instead of excluding it. These checks read the committed files, so a
//! change that drops one of them fails `cargo test` before reaching the stacks.
use std::fs;
use std::path::Path;

fn read(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// The block of service `name` in a compose file (its lines up to the next service).
fn service<'a>(compose: &'a str, name: &str) -> &'a str {
    let header = format!("\n  {name}:\n");
    let start = compose
        .find(&header)
        .unwrap_or_else(|| panic!("service {name} is missing"));
    let body = &compose[start + header.len()..];
    let end = body
        .match_indices("\n  ")
        .find(|(index, _)| {
            body[index + 3..]
                .chars()
                .next()
                .is_some_and(|c| c != ' ' && c != '#')
        })
        .map_or(body.len(), |(index, _)| index);
    &body[..end]
}

#[test]
fn mair_661_the_integration_stack_runs_the_websocket_journey_after_newman() {
    let compose = read("docker-compose-integration.yml");
    let journey = service(&compose, "ws-journey");
    assert!(
        journey.contains("profiles: [\"journey\"]"),
        "ws-journey must stay out of `up` (profile journey):\n{journey}"
    );
    assert!(journey.contains("./tests/websocket:/journey:ro"));
    assert!(journey.contains("node\", \"/journey/journey.mjs\""));
    for token in ["JWT_ADMIN", "JWT_AGENT", "JWT_THIRD"] {
        assert!(journey.contains(token), "ws-journey needs {token}");
    }

    let script = read("integration_test.sh");
    let wait = script
        .find("wait \"$SERVICE_NAME\"")
        .expect("integration_test.sh waits for newman");
    let run = script
        .find("--profile journey run --rm ws-journey")
        .expect("integration_test.sh runs the journey");
    assert!(run > wait, "the journey runs once newman is done");
    assert!(
        script.contains("--profile journey down -v"),
        "the teardown removes the journey containers too"
    );
}

#[test]
fn mair_661_the_journey_checks_members_outsider_and_eviction() {
    let journey = read("tests/websocket/journey.mjs");
    for expected in [
        "/api/v1/ws",
        "NEW_MSG",
        "chat_id",
        "message_id",
        "C, not a member, receives nothing",
        "A, the author, receives nothing",
        "const EVICTED = 4008",
        "const MAX_CONNECTIONS = 5",
        "process.exit(failures === 0 ? 0 : 1)",
    ] {
        assert!(journey.contains(expected), "journey.mjs lost `{expected}`");
    }
}

#[test]
fn mair_661_k6_holds_websockets_under_load_with_strict_thresholds() {
    let k6 = read("load-test.js");
    for expected in [
        "import ws from 'k6/ws'",
        "exec: 'websocketScenario'",
        "export function websocketScenario(",
        "ws_frame_delivery: [`p(95)<${WS_DELIVERY_BUDGET_MS}`]",
        "ws_frames_lost: ['count==0']",
        "'ws_connecting{scenario:websocket}'",
        "checks: ['rate==1']",
        "http_req_failed: ['rate==0']",
        "dropped_iterations: ['count==0']",
    ] {
        assert!(k6.contains(expected), "load-test.js lost `{expected}`");
    }
    let ci = k6
        .lines()
        .find(|line| line.trim_start().starts_with("ci: {"))
        .expect("ci profile");
    assert!(ci.contains("wsVus:"), "the ci profile sets wsVus: {ci}");
}

#[test]
fn mair_661_zap_scans_the_websocket_route() {
    for line in read(".zap/rules.tsv").lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.get(1) == Some(&"OUTOFSCOPE") {
            assert!(
                !fields[2].contains("ws"),
                "/api/v1/ws must stay in the ZAP scope: {line}"
            );
        }
    }
}
