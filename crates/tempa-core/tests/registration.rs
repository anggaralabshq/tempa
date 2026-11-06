//! M3-lite: the `tempa` MCP server registered in every agent config.
mod common;
use common::{read, valid_repo, write};
use serde_json::json;

// ---- Task 4: registration ----

const MCP_CONFIGS: [&str; 5] = [
    ".mcp.json",
    ".cursor/mcp.json",
    ".gemini/settings.json",
    "opencode.json",
    ".codex/config.toml",
];

fn set_serve(p: &std::path::Path, serve: bool) {
    let t = read(p, "tempa.toml")
        .replace("serve = true", "serve = X")
        .replace("serve = false", "serve = X");
    assert!(t.contains("serve = X"), "{t}");
    write(
        p,
        "tempa.toml",
        &t.replace("serve = X", &format!("serve = {serve}")),
    );
}

#[test]
fn sync_registers_tempa_server_everywhere() {
    let d = valid_repo();
    let p = d.path();
    for f in MCP_CONFIGS {
        let t = read(p, f);
        assert!(
            t.contains("\"tempa\"") || t.contains("[mcp_servers.tempa]"),
            "{f}: {t}"
        );
        assert!(t.contains("mcp"), "{f}: {t}");
    }
    let claude: serde_json::Value = serde_json::from_str(&read(p, ".mcp.json")).unwrap();
    assert_eq!(
        claude["mcpServers"]["tempa"],
        json!({"command": "tempa", "args": ["mcp"]})
    );
    let oc: serde_json::Value = serde_json::from_str(&read(p, "opencode.json")).unwrap();
    assert_eq!(oc["mcp"]["tempa"]["command"], json!(["tempa", "mcp"]));
    let settings: serde_json::Value =
        serde_json::from_str(&read(p, ".claude/settings.json")).unwrap();
    assert_eq!(settings["enabledMcpjsonServers"], json!(["tempa"]));
}

#[test]
fn serve_false_removes_entry_keeps_other_keys() {
    let d = valid_repo();
    let p = d.path();
    write(p, ".tempa/mcp.toml", "[servers.db]\ncommand = \"db-mcp\"\n");
    tempa_core::sync::sync(p).unwrap();
    set_serve(p, false);
    tempa_core::sync::sync(p).unwrap();
    for f in MCP_CONFIGS {
        let t = read(p, f);
        assert!(
            !t.contains("\"tempa\"") && !t.contains("mcp_servers.tempa"),
            "{f}: {t}"
        );
        assert!(t.contains("db"), "{f}: {t}");
    }
    let settings = read(p, ".claude/settings.json");
    assert!(!settings.contains("enabledMcpjsonServers"), "{settings}");
    assert!(settings.contains("deny"), "{settings}");
}

#[test]
fn serve_false_without_servers_drops_mcp_only_files() {
    let d = valid_repo();
    let p = d.path();
    set_serve(p, false);
    tempa_core::sync::sync(p).unwrap();
    assert!(!p.join(".mcp.json").exists());
    assert!(!p.join(".cursor/mcp.json").exists());
}

#[test]
fn user_server_named_tempa_wins() {
    let d = valid_repo();
    let p = d.path();
    write(
        p,
        ".tempa/mcp.toml",
        "[servers.tempa]\ncommand = \"/opt/tempa-dev\"\nargs = [\"mcp\", \"--verbose\"]\n",
    );
    tempa_core::sync::sync(p).unwrap();
    let v: serde_json::Value = serde_json::from_str(&read(p, ".mcp.json")).unwrap();
    assert_eq!(v["mcpServers"]["tempa"]["command"], json!("/opt/tempa-dev"));
    let settings = read(p, ".claude/settings.json");
    assert!(!settings.contains("enabledMcpjsonServers"), "{settings}");
}

#[test]
fn import_does_not_copy_injected_server() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    common::git_init(p);
    write(p, "CLAUDE.md", "# x\n\n## Stack\n\nrust\n");
    write(
        p,
        ".mcp.json",
        r#"{"mcpServers":{"tempa":{"command":"tempa","args":["mcp"]},"db":{"command":"db-mcp"}}}"#,
    );
    tempa_core::import::import(p, true).unwrap();
    let m = read(p, ".tempa/mcp.toml");
    assert!(m.contains("db"), "{m}");
    assert!(!m.contains("tempa"), "{m}");
}

#[test]
fn doctor_lifecycle_mcp() {
    let d = valid_repo();
    let p = d.path();
    let r = tempa_core::doctor::doctor(p).unwrap();
    assert!(r.rows.iter().all(|row| row.lifecycle == "mcp"));
    set_serve(p, false);
    let r = tempa_core::doctor::doctor(p).unwrap();
    assert!(r.rows.iter().all(|row| row.lifecycle == "off"));
}

#[test]
fn init_agents_md_names_the_mcp_tools() {
    let d = tempfile::tempdir().unwrap();
    common::git_init(d.path());
    tempa_core::init::init(d.path()).unwrap();
    let a = read(d.path(), "AGENTS.md");
    let second = a.lines().nth(2).unwrap_or_default();
    assert!(
        second.starts_with("Agents: use the tempa_* MCP tools"),
        "{a}"
    );
}

// Pilot: OpenCode writes "$schema" into opencode.json when it runs; tempa renders it
// so running OpenCode does not create drift.
#[test]
fn opencode_json_carries_schema() {
    let d = valid_repo();
    let v: serde_json::Value = serde_json::from_str(&read(d.path(), "opencode.json")).unwrap();
    assert_eq!(v["$schema"], json!("https://opencode.ai/config.json"));
}

// I1: an MCP config the user made by hand (never written by tempa) is reported as
// foreign with the way out, and `tempa import --yes` absorbs it.
#[test]
fn hand_made_mcp_config_is_foreign_and_import_absorbs_it() {
    let d = valid_repo();
    let p = d.path();
    set_serve(p, false);
    tempa_core::sync::sync(p).unwrap();
    write(
        p,
        ".mcp.json",
        r#"{"mcpServers":{"db":{"command":"db-mcp"}}}"#,
    );
    set_serve(p, true);
    let f = tempa_core::sync::drift_findings(&tempa_core::sync::drift(p).unwrap());
    let mine: Vec<_> = f
        .iter()
        .filter(|f| f.path.as_deref() == Some(".mcp.json"))
        .collect();
    assert_eq!(mine.len(), 1, "{f:?}");
    assert_eq!(mine[0].code, "drift.foreign");
    assert!(
        mine[0].message.contains("tempa import --yes"),
        "{}",
        mine[0].message
    );
    tempa_core::import::import(p, true).unwrap();
    assert!(tempa_core::sync::drift(p).unwrap().is_empty());
    let v: serde_json::Value = serde_json::from_str(&read(p, ".mcp.json")).unwrap();
    assert!(
        v["mcpServers"]["db"].is_object() && v["mcpServers"]["tempa"].is_object(),
        "{v}"
    );
}

// I2: import must not copy the injected enabledMcpjsonServers entry into the base.
#[test]
fn import_strips_injected_enabled_server_from_claude_base() {
    let d = valid_repo();
    let p = d.path();
    let mut v: serde_json::Value = serde_json::from_str(&read(p, ".claude/settings.json")).unwrap();
    v["model"] = json!("opus");
    write(
        p,
        ".claude/settings.json",
        &serde_json::to_string_pretty(&v).unwrap(),
    );
    tempa_core::import::import(p, true).unwrap();
    let base = read(p, ".tempa/targets/.claude/settings.json");
    assert!(!base.contains("enabledMcpjsonServers"), "{base}");
    set_serve(p, false);
    tempa_core::sync::sync(p).unwrap();
    let s = read(p, ".claude/settings.json");
    assert!(
        s.contains("opus") && !s.contains("enabledMcpjsonServers"),
        "{s}"
    );
}
