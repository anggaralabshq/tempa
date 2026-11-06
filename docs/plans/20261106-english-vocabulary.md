---
task_id: 20261106-english-vocabulary
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Make English the default contract vocabulary (required headings, changes fields, pr-body sections) while repos written with the old Indonesian names keep passing.

## Steps

- `contract::same_name` with a LEGACY table maps each English name to its Indonesian predecessor; used by heading checks, change-field checks, `tempa_plan` and `md::set_section`.
- `config.rs` heading defaults and `CHANGE_FIELDS` move to English.
- Plan and decision templates are built from the configured headings (`headings_only`) instead of fixed template files.
- `pr_body` uses English section names and reads the summary from the first plan heading, legacy or not.
- Update tests and `scripts/e2e.sh` (also translate its messages).

## Files touched

crates/tempa-core/src/{config,contract,mcp,md,task}.rs, crates/tempa-core/src/templates/{plan,decision}.md, crates/tempa-core/tests/*.rs, scripts/e2e.sh

## Risks

Existing repos keep Indonesian headings in tempa.toml or none at all; the legacy aliases keep both passing. A repo that set custom headings is unaffected. Old CHANGES.md entries with Ringkasan/Alasan still validate.
