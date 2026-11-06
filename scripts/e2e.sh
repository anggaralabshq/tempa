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
echo "== step 1: fixture + git"
cp -R "$FIXTURE"/. .
git init -q -b main
git config user.name e2e && git config user.email e2e@x
git add -A && git commit -qm "initial"

echo "== step 2: import"
expect_code 0 tempa import
[ ! -f tempa.toml ] || fail "dry-run wrote tempa.toml"
expect_code 0 tempa import --yes
expect_code 0 tempa sync --check

echo "== step 3: document contract"
expect_code 1 tempa check --contract-only
echo "$OUT" | grep -q "contract.missing_heading" || fail "expected contract.missing_heading:\n$OUT"
grep -qi "^## Conventions" AGENTS.md || printf '\n## Conventions\n\nFollow docs/style.md.\n' >> AGENTS.md
printf '\n## Off-limits\n\nlegacy/\n' >> AGENTS.md
printf '# Harness\n\n## Definition of done\n\ncargo test passes.\n\n## Gates\n\ntempa check.\n' > HARNESS.md
git add -A && git commit -qm "tempa contract"
expect_code 0 tempa check --contract-only

echo "== step 4: drift blocked, hook syncs"
echo "direct edit" >> CLAUDE.md
expect_code 1 tempa sync --check
echo "$OUT" | grep -q "drift.modified" || fail "expected drift.modified:\n$OUT"
git add CLAUDE.md
expect_code 1 git commit -qm "edit a target directly"
git reset -q CLAUDE.md && git checkout -q CLAUDE.md
CMD=$(cd .tempa/commands && find . -name "*.md" -type f | sed "s|^./||" | sort | head -1)
printf '\nNew rule from e2e.\n' >> ".tempa/commands/$CMD"
git add ".tempa/commands/$CMD"
expect_code 0 git commit -qm "change a command"
git show --name-only --format= HEAD | grep -q ".claude/commands/$CMD" || fail "commit lacks the updated .claude/commands/$CMD"
expect_code 0 tempa sync --check

echo "== step 5: task cycle"
expect_code 0 tempa task new add-invoice
PLAN=$(ls docs/plans/*-add-invoice.md)
mkdir -p src && echo "pub fn invoice() {}" > src/invoice.rs
git add -A && git commit -qm "invoice wip"
expect_code 1 tempa check --contract-only --base main
echo "$OUT" | grep -q "task.no_change_entry" || fail "expected task.no_change_entry:\n$OUT"
for h in Goal Steps "Files touched" Risks; do
  python3 - "$PLAN" "$h" <<'PY'
import sys
p, h = sys.argv[1], sys.argv[2]
s = open(p).read().replace(f"## {h}\n", f"## {h}\n\ntext for {h}.\n", 1)
open(p, "w").write(s)
PY
done
expect_code 0 tempa task approve
expect_code 0 tempa log --summary s --reason r --agent "codex / gpt-5"
git add -A && git commit -qm "invoice done"
expect_code 0 tempa check --contract-only --base main

echo "== step 6: pr-body"
expect_code 0 tempa pr-body
echo "$OUT" | grep -q "## Agent + model" || fail "pr-body lacks '## Agent + model':\n$OUT"

echo "== step 7: doctor"
expect_code 0 tempa doctor
echo "$OUT" | grep -q "^codex" || fail "doctor lacks the codex row:\n$OUT"

echo "== step 8: two clones, lock committed, edits kept"
git checkout -q main && git merge -q --no-edit "$(git rev-parse --abbrev-ref @{-1})"
git init -q --bare "$WORK/origin.git" && git push -q --no-verify "$WORK/origin.git" main
git clone -q "$WORK/origin.git" "$WORK/B" && (cd "$WORK/B" && git config core.hooksPath .tempa/hooks)
printf -- '---\ndescription: R\n---\nreview version A\n' > ".tempa/commands/review.md"
git add -A && git commit -qm "A changes review"
[ -z "$(git status --porcelain)" ] || fail "lock/targets not committed:\n$(git status --porcelain)"
git push -q --no-verify "$WORK/origin.git" main
cd "$WORK/B" && git pull -q origin main
printf -- '---\ndescription: R\n---\nreview version A\n\nStep B.\n' > ".tempa/commands/review.md"
git add -A
expect_code 0 git commit -qm "B changes review"
grep -q "Step B." .claude/commands/review.md || fail "B's edit is missing from the target"
expect_code 0 tempa sync --check

echo "e2e OK"
