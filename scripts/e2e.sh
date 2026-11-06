#!/usr/bin/env bash
# End-to-end: import a Claude-heavy repo, keep it in sync, run one task to a green contract.
# Usage: scripts/e2e.sh <path-to-tempa-binary>
set -euo pipefail

BIN=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
FIXTURE=$(cd "$(dirname "$0")/.." && pwd)/crates/tempa-core/tests/fixtures/claude-heavy
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
export PATH="$(dirname "$BIN"):$PATH"   # hooks call `tempa` from PATH
export GIT_AUTHOR_NAME=e2e GIT_AUTHOR_EMAIL=e2e@x GIT_COMMITTER_NAME=e2e GIT_COMMITTER_EMAIL=e2e@x
unset BITBUCKET_BRANCH BITBUCKET_PR_ID || true

fail() { echo "FAIL: $*" >&2; exit 1; }
expect_code() { # <code> <cmd...>  → runs cmd, stores output in $OUT
  local want=$1; shift
  set +e; OUT=$("$@" 2>&1); local got=$?; set -e
  [ "$got" = "$want" ] || fail "'$*' exit $got, expected $want:\n$OUT"
}

cd "$WORK"
echo "== langkah 1: fixture + git"
cp -R "$FIXTURE"/. .
git init -q -b main
git config user.name e2e && git config user.email e2e@x
git add -A && git commit -qm "awal"

echo "== langkah 2: import"
expect_code 0 tempa import
[ ! -f tempa.toml ] || fail "dry-run menulis tempa.toml"
expect_code 0 tempa import --yes
expect_code 0 tempa sync --check

echo "== langkah 3: kontrak dokumen"
expect_code 1 tempa check --contract-only
echo "$OUT" | grep -q "contract.missing_heading" || fail "harus ada contract.missing_heading:\n$OUT"
grep -qi "^## Konvensi" AGENTS.md || printf '\n## Konvensi\n\nIkuti docs/style.md.\n' >> AGENTS.md
printf '\n## Area terlarang\n\nlegacy/\n' >> AGENTS.md
printf '# Harness\n\n## Definisi selesai\n\ncargo test lulus.\n\n## Gate yang berlaku\n\ntempa check.\n' > HARNESS.md
git add -A && git commit -qm "kontrak tempa"
expect_code 0 tempa check --contract-only

echo "== langkah 4: drift diblok, sync otomatis lewat hook"
echo "edit langsung" >> CLAUDE.md
expect_code 1 tempa sync --check
echo "$OUT" | grep -q "drift.modified" || fail "harus drift.modified:\n$OUT"
git add CLAUDE.md
expect_code 1 git commit -qm "edit target langsung"
git reset -q CLAUDE.md && git checkout -q CLAUDE.md
printf '\nAturan baru dari e2e.\n' >> AGENTS.md
git add AGENTS.md
expect_code 0 git commit -qm "ubah AGENTS.md"
git show --name-only --format= HEAD | grep -q ".cursor/rules/tempa.mdc" || fail "commit tidak berisi tempa.mdc ter-update"
expect_code 0 tempa sync --check

echo "== langkah 5: siklus task"
expect_code 0 tempa task new add-invoice
PLAN=$(ls docs/plans/*-add-invoice.md)
mkdir -p src && echo "pub fn invoice() {}" > src/invoice.rs
git add -A && git commit -qm "invoice wip"
expect_code 1 tempa check --contract-only --base main
echo "$OUT" | grep -q "task.no_change_entry" || fail "harus task.no_change_entry:\n$OUT"
for h in Tujuan Langkah "File yang disentuh" Risiko; do
  python3 - "$PLAN" "$h" <<'PY'
import sys
p, h = sys.argv[1], sys.argv[2]
s = open(p).read().replace(f"## {h}\n", f"## {h}\n\nisi {h}.\n", 1)
open(p, "w").write(s)
PY
done
expect_code 0 tempa task approve
expect_code 0 tempa log --summary s --reason r --agent "codex / gpt-5"
git add -A && git commit -qm "invoice selesai"
expect_code 0 tempa check --contract-only --base main

echo "== langkah 6: pr-body"
expect_code 0 tempa pr-body
echo "$OUT" | grep -q "## Agent + model" || fail "pr-body tanpa '## Agent + model':\n$OUT"

echo "== langkah 7: doctor"
expect_code 0 tempa doctor
echo "$OUT" | grep -q "^codex" || fail "doctor tanpa baris codex:\n$OUT"

echo "e2e OK"
