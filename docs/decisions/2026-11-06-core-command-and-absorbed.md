## Konteks

The M2 review found two gaps that need tempa-core API: the desktop app (no console on Windows) flashed a console window for every git call, and the Absorb dialog could not list the targets an import overwrites.

## Opsi

1. Add `tempa_core::command(program)` (Command with CREATE_NO_WINDOW on Windows) and use it in gitx; add `ImportReport.absorbed`. 2. Duplicate the flag logic in the app and re-derive absorbed targets from drift there: two copies of the rules, against the spec (app and CLI share tempa-core).

## Keputusan

Option 1. The CLI behaves the same: its git calls capture output, so a missing console changes nothing for it.

## Konsekuensi

Two small additions to the public API of tempa-core. Every new subprocess in core or app should use `command`.

## Agent + model

claude-code / claude-opus-5-5
