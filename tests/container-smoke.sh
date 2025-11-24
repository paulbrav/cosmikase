#!/usr/bin/env bash
set -euo pipefail

ENGINE="${ENGINE:-podman}"
IMAGE="${IMAGE:-ubuntu:24.04}"
REPO_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="/workspace/tests/test-config.yaml"

if ! command -v "$ENGINE" >/dev/null 2>&1; then
  echo "Container engine $ENGINE not found (set ENGINE=docker or install podman)." >&2
  exit 1
fi

echo "Starting smoke test in container using $ENGINE and $IMAGE"

$ENGINE run --rm -t \
  --privileged \
  -v "$REPO_DIR":/workspace \
  "$IMAGE" \
  bash -c "\
    apt-get update && apt-get install -y sudo curl rsync unzip python3 python3-pip git && \
    pip3 install pyyaml && \
    cd /workspace && \
    CONFIG_FILE=$CONFIG bash ./install.sh"

echo "Smoke test finished. Review output above for errors."
