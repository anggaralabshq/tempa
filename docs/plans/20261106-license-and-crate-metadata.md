---
task_id: 20261106-license-and-crate-metadata
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Make tempa publishable: dual MIT OR Apache-2.0 license (owner's choice) and crates.io metadata. The name tempa is taken on crates.io, so the CLI ships as tempa-cli (binary still tempa) with tempa-core as its library.

## Langkah

1. LICENSE-MIT, LICENSE-APACHE (text from apache.org); README License section.
2. Workspace license, repository, keywords, categories; per-crate description; tempa-cli readme; tempa-core path dependency gains a version.
3. cargo publish -p tempa-core --dry-run passes (67 KB).

## File yang disentuh

LICENSE-MIT, LICENSE-APACHE, README.md, Cargo.toml, crates/tempa-cli/Cargo.toml, crates/tempa-core/Cargo.toml.

## Risiko

Copyright line says "The tempa contributors"; change it if the owner wants a legal entity. Nothing is published by this change.
