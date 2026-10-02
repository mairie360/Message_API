#!/usr/bin/env bash
# Sourced by integration_test.sh, security_test.sh and performance_test.sh (MAIR-428).
#
# Every run of a test stack gets its own random JWT_SECRET (unless one is exported), and the
# administrator token the scanners send is forged from it here, at run time: no secret and no
# token valid on any instance is committed. API_lib refuses to start with a short or well-known
# JWT_SECRET, so the stacks could not use a fixed placeholder anyway.

b64url() {
    openssl base64 -A | tr '+/' '-_' | tr -d '='
}

# HS256 JWT with the claims API_lib reads (`sub`, `role`, `exp`), valid for two hours.
# Usage: forge_jwt <user id> <role>
forge_jwt() {
    local header payload signature
    header=$(printf '{"alg":"HS256","typ":"JWT"}' | b64url)
    payload=$(printf '{"sub":"%s","role":"%s","exp":%d}' "$1" "$2" "$(($(date +%s) + 7200))" | b64url)
    signature=$(printf '%s.%s' "$header" "$payload" | openssl dgst -sha256 -hmac "$JWT_SECRET" -binary | b64url)
    printf '%s.%s.%s' "$header" "$payload" "$signature"
}

if [ -z "${JWT_SECRET:-}" ]; then
    JWT_SECRET=$(openssl rand -hex 32)
fi
export JWT_SECRET
# User 1 is the Admin seeded by liquibase.
ADMIN_JWT=$(forge_jwt 1 admin)
export ADMIN_JWT
