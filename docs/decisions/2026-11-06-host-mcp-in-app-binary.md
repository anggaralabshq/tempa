## Konteks

M3 injects a tempa MCP server into every ACP session. Its tools must follow the app's rules, and a harness check must be recorded by the host, not reported by the agent. The lifecycle also needs states after approval (implement, verify, record).

## Opsi

1. Inject the installed `tempa mcp` CLI: depends on the CLI on PATH and its version, and cannot record results for the app. 2. Run the server in the app process and bridge it over a socket (Termul host_mcp): more moving parts. 3. Inject the app binary itself in `--mcp <root> --verify <file>` mode, serving tempa_core::mcp and recording tempa_check {harness:true} into verify.json.

## Keputusan

Option 3. tempa-core gains `task::STATES` and `task::set_state` so the host moves the plan's state with the same code the CLI uses.

## Konsekuensi

Rules always match the app version. The recording is best effort (an agent could edit verify.json); CI stays the final gate. Plans may now carry states implement/verify/record.

## Agent + model

claude-code / claude-opus-5-5
