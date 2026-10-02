#!/bin/bash
set -e

# Same directory as the WORKDIR of development.Dockerfile
cd /usr/src/message

exec cargo watch --poll -w src -i target -x run
