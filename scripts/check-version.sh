#!/bin/sh
# Fail unless TAG (v1.2.3) matches the workspace version and the version tempa-cli
# requires of tempa-core. Run by the release workflow before anything is published.
set -eu
TAG=$1
ROOT=$(cd "$(dirname "$0")/.." && pwd)
ws=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -1)
core=$(sed -n 's/^tempa-core = .*version = "\([^"]*\)".*/\1/p' "$ROOT/crates/tempa-cli/Cargo.toml")
[ "$TAG" = "v$ws" ] || { echo "tag $TAG does not match Cargo.toml version $ws" >&2; exit 1; }
[ "$core" = "$ws" ] || { echo "tempa-cli requires tempa-core $core, workspace is $ws" >&2; exit 1; }
echo "version $ws ok"
