---
task_id: 20261106-public-readiness
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Make the repo ready to be public: contributor and security policies, honest positioning, and an install section that is true before the first release.

## Steps

- Add CONTRIBUTING.md (workflow through tempa, gates, PR size, license) and SECURITY.md (private advisories, scope).
- README: note that brew/crates.io/Releases installs start at v0.1.0; add "How tempa compares" naming rulesync, ruler, ai-rulez and APM; say `app/` is frozen.

## Files touched

CONTRIBUTING.md, SECURITY.md, README.md

## Risks

Docs only. Competitor coverage numbers drift; the README gives a range, not exact counts.
