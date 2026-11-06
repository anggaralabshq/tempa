## Konteks

tempa grew from contract sync (M1) into an agent host (M3: ACP chat, tool policy, worktrees). The owner judged that a drift from the original idea: one contract (AGENTS.md + .tempa/ + tempa.toml) aligned with each coding agent's own format.

## Opsi

1. Keep building M3. 2. Freeze M3 on a branch and keep its code on main unused. 3. Freeze M3 on branch m3 and remove its code from main. 4. Delete M3 entirely.

## Keputusan

Option 3. Branch m3 keeps all M3 work (Tasks 1-3 merged there, Task 4 WIP). main returns to the M2 app; the desktop app is frozen as-is. Lifecycle commands stay in the CLI but are optional. Next: add agents (Copilot first), verify target formats against official docs, release and distribution, public readiness.

## Konsekuensi

Resuming M3 means merging m3 again. The CLI is the product; the app gets no new features for now. CI runs once per milestone.

## Agent + model

claude-code / claude-opus-5-5
