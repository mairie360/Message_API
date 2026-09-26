//! Regression guard for MAIR-229: the production image must not run as root.
//!
//! Reads the production `Dockerfile` and checks its last (runtime) stage: it must be built
//! from the distroless `nonroot` variant and declare a numeric, non-zero `USER` so that
//! Kubernetes can enforce `runAsNonRoot: true`.

use std::path::Path;

/// Returns the instructions of the last build stage (from its `FROM` line onwards).
fn runtime_stage(dockerfile: &str) -> Vec<String> {
    let lines: Vec<String> = dockerfile
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();
    let start = lines
        .iter()
        .rposition(|line| line.to_ascii_uppercase().starts_with("FROM "))
        .expect("Dockerfile has no FROM instruction");
    lines[start..].to_vec()
}

fn production_runtime_stage() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Dockerfile");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    runtime_stage(&content)
}

#[test]
fn runtime_stage_uses_distroless_nonroot_image() {
    let stage = production_runtime_stage();
    let from = &stage[0];
    assert!(
        from.contains("gcr.io/distroless/") && from.contains(":nonroot"),
        "runtime stage must use a distroless `:nonroot` image, got `{from}`"
    );
}

#[test]
fn runtime_stage_declares_numeric_non_root_user() {
    let stage = production_runtime_stage();
    let user = stage
        .iter()
        .rev()
        .find(|line| line.to_ascii_uppercase().starts_with("USER "))
        .expect("runtime stage must declare a USER instruction");
    let uid = user[5..].trim().split(':').next().unwrap_or_default();
    let uid: u32 = uid
        .parse()
        .unwrap_or_else(|_| panic!("USER must be numeric for runAsNonRoot, got `{user}`"));
    assert_ne!(uid, 0, "runtime stage must not run as root");
}
