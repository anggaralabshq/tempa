## Context

tempa shipped with Indonesian contract vocabulary (Tujuan, Ringkasan, ...). The CLI is going public and every other user-facing text is English.

## Options

1. Keep Indonesian defaults. 2. Switch to English and break existing repos. 3. Switch to English with legacy Indonesian aliases accepted everywhere a name is matched.

## Decision

Option 3. Defaults, templates, CHANGES fields and pr-body sections are English; contract::same_name accepts the Indonesian predecessor of each English name.

## Consequences

Existing repos pass without migration. New plans and decision records use English headings. The alias table is fixed; a repo with custom headings matches them exactly as before.

## Agent + model

claude-code / claude-opus-5-5
