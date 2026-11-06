---
task_id: 20261106-readme-banner
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Make the README read like a product page: a banner, badges, a call-to-action row, a 3-step quick start and an architecture diagram that shows one source synced to six agents.

## Steps

1. Add banner and diagram PNGs (light and dark) under `docs/assets/`, rendered from the Remotion project.
2. README: banner with `<picture>` light/dark, CTA links, badges, diagram under the intro, quick start as three numbered commands before the full install options.

## Files touched

- `README.md`
- `docs/assets/banner-{light,dark}.png`, `docs/assets/architecture-{light,dark}.png`

## Risks

- Images go stale when agents or commands change; regenerate them together with `docs/target-matrix.md`.
- Binary assets add ~0.9 MB to the repository.
