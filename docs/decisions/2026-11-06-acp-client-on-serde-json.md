## Konteks

The app talks to five coding agents over ACP (newline-delimited JSON-RPC on stdio, both directions). It needs initialize, session/new with the tempa MCP server, prompt, cancel, the session/update notification, the request_permission request, and model selection via config options.

## Opsi

1. Crate `agent-client-protocol` (3.x/5.x): typed schema, but its connection needs a local non-Send async executor; Termul vendors and patches it. 2. A small client on serde_json and threads, for the subset of methods tempa uses, as with the MCP server (docs/decisions/2026-11-06-mcp-server-on-serde-json.md).

## Keputusan

Option 2. Wire facts checked against the official schema (schema/v1/schema.json, meta.json). A Node fake agent (app/src-tauri/tests/fake-agent.mjs) drives the tests; agents run in their own process group (Unix) or are ended with taskkill /T (Windows) so npx children never outlive a session.

## Konsekuensi

No new dependency. New ACP fields have to be read by hand; the schema is the reference when an adapter misbehaves. Tests need node on PATH.

## Agent + model

claude-code / claude-opus-5-5
