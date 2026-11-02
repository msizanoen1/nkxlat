#!/usr/bin/env bash
set -euo pipefail

mkdir -p container-build
echo '*' >container-build/.gitignore

podman build --iidfile=container-build/image -f scripts/Dockerfile.build

podman run \
    --rm \
    --security-opt label=disable \
    -v "$(pwd):/src" \
    -w /src \
    -e CARGO_HOME=/src/container-build/cargo \
    -e CARGO_TARGET_DIR=/src/container-build/target \
    "$(cat container-build/image)" \
    cargo build --release --features libbpf-rs/vendored
