#!/usr/bin/env bash

# Runs the Postman collection committed under tests/postman/ with newman
# (docker-compose-integration.yml). The exit code of the `newman` service is the exit code of
# this script, so CI fails on the first failing request (`--bail`). No Postman account or API
# key is needed.
#
# The API under test is the image named by IMAGE_REF. CI sets it to the published
# ghcr.io/mairie360/message-api:dev-<sha> image (the one promoted to staging and prod); when it is
# empty (local usage) the image is built from development.Dockerfile as message-api:local.

COMPOSE_FILE="docker-compose-integration.yml"
SERVICE_NAME="newman"

if [ -z "${IMAGE_REF:-}" ]; then
    echo "==> [0/4] IMAGE_REF is empty: building message-api:local from development.Dockerfile..."
    IMAGE_REF="message-api:local"
    docker build -f development.Dockerfile -t "$IMAGE_REF" . || exit 1
fi
export IMAGE_REF
echo "==> API image under test: $IMAGE_REF"

# Fresh Ed25519 key pair for this run: the API gets its public key, the runners tokens signed with
# it like Core signs them (MAIR-428, MAIR-680, test_secrets.sh).
# shellcheck source=test_secrets.sh
source "$(dirname "$0")/test_secrets.sh" || exit 1
# The collection's callers (the Postman sandbox cannot sign EdDSA): the seeded Admin (user 1), plain
# users 2 and 3 (init-test.sql), a token claiming Core's kid but signed with another key, and an
# expired one.
JWT_ADMIN="$ADMIN_JWT"
JWT_AGENT="$(sign_jwt 2 user 3600)" || exit 1
JWT_THIRD="$(sign_jwt 3 user 3600)" || exit 1
JWT_FORGED="$(sign_jwt 1 admin 3600 "$(openssl genpkey -algorithm ed25519 2>/dev/null)")" || exit 1
JWT_EXPIRED="$(sign_jwt 1 admin -3600)" || exit 1
export JWT_ADMIN JWT_AGENT JWT_THIRD JWT_FORGED JWT_EXPIRED

echo "==> [1/4] Starting the stack and the newman collection..."
# Fresh database on every run (MAIR-474): a volume left by a previous run would hand the
# stack its rows (rows the scan deleted, the previous volume seed) instead of the seed under test.
docker compose -f "$COMPOSE_FILE" down -v --remove-orphans > /dev/null 2>&1
docker compose -f "$COMPOSE_FILE" up -d

echo "==> [2/4] Waiting for the newman collection to finish..."
docker compose -f "$COMPOSE_FILE" wait "$SERVICE_NAME"
EXIT_CODE=$?

echo "==> [3/4] Report (logs)..."
docker compose -f "$COMPOSE_FILE" logs "$SERVICE_NAME"

echo "==> [4/4] Cleaning up the containers..."
docker compose -f "$COMPOSE_FILE" down -v

echo "----------------------------------------"
echo "Final exit code: $EXIT_CODE"
echo "----------------------------------------"

exit $EXIT_CODE
