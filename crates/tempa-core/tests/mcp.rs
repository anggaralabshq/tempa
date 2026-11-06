//! M3-lite: `tempa mcp` protocol and tools.
mod common;
use common::{git, read, valid_repo, write};
use serde_json::{Value, json};
use tempa_core::mcp::{PROTOCOL_VERSIONS, handle, handle_line};
use tempa_core::task::task_new;

fn req(method: &str, params: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
}

fn call(root: &std::path::Path, name: &str, args: Value) -> (bool, String) {
    let r = handle(
        root,
        &req("tools/call", json!({"name": name, "arguments": args})),
    )
    .unwrap();
    let res = &r["result"];
    (
        res["isError"].as_bool().unwrap_or(false),
        res["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
    )
}

// ---- Task 1: protocol ----

#[test]
fn initialize_negotiates_version() {
    let d = valid_repo();
    let r = handle(d.path(), &req("initialize", json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "1"}}))).unwrap();
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(r["result"]["serverInfo"]["name"], "tempa");
    assert!(r["result"]["capabilities"]["tools"].is_object());
    let r = handle(
        d.path(),
        &req("initialize", json!({"protocolVersion": "1999-01-01"})),
    )
    .unwrap();
    assert_eq!(r["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
}

#[test]
fn notifications_get_no_reply() {
    let d = valid_repo();
    assert!(
        handle(
            d.path(),
            &json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
        )
        .is_none()
    );
    assert!(
        handle(
            d.path(),
            &json!({"jsonrpc": "2.0", "method": "notifications/whatever"})
        )
        .is_none()
    );
}

#[test]
fn ping_returns_empty_result() {
    let d = valid_repo();
    assert_eq!(
        handle(d.path(), &req("ping", json!({}))).unwrap()["result"],
        json!({})
    );
}

#[test]
fn unknown_method_is_32601() {
    let d = valid_repo();
    assert_eq!(
        handle(d.path(), &req("nope", json!({}))).unwrap()["error"]["code"],
        -32601
    );
}

#[test]
fn garbage_is_32700_and_batch_is_32600() {
    let d = valid_repo();
    let r: Value = serde_json::from_str(&handle_line(d.path(), "{not json").unwrap()).unwrap();
    assert_eq!(r["error"]["code"], -32700);
    assert_eq!(r["id"], Value::Null);
    let r: Value = serde_json::from_str(&handle_line(d.path(), "[]").unwrap()).unwrap();
    assert_eq!(r["error"]["code"], -32600);
    assert!(handle_line(d.path(), "   ").is_none());
}

#[test]
fn tools_list_names() {
    let d = valid_repo();
    let r = handle(d.path(), &req("tools/list", json!({}))).unwrap();
    let tools = r["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        [
            "tempa_context",
            "tempa_status",
            "tempa_plan",
            "tempa_decide",
            "tempa_log_change",
            "tempa_check",
            "tempa_pr_body"
        ]
    );
    assert!(
        tools
            .iter()
            .all(|t| t["inputSchema"]["type"] == "object" && t["description"].is_string())
    );
}

#[test]
fn unknown_tool_is_error_result() {
    let d = valid_repo();
    let (err, text) = call(d.path(), "tempa_nope", json!({}));
    assert!(err && text.contains("tempa_nope"), "{text}");
}

// ---- Task 2: read tools ----

#[test]
fn context_serves_commands_codex_cannot_read() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: Review diff\n---\nCek keamanan dulu.\n",
    );
    let (err, text) = call(
        d.path(),
        "tempa_context",
        json!({"kind": "commands", "name": "review"}),
    );
    assert!(!err, "{text}");
    assert!(
        text.contains("Cek keamanan dulu.") && text.contains("Review diff"),
        "{text}"
    );
    let (err, all) = call(d.path(), "tempa_context", json!({}));
    assert!(
        !err && all.contains("## Stack") && all.contains("review"),
        "{all}"
    );
    let (err, _) = call(
        d.path(),
        "tempa_context",
        json!({"kind": "commands", "name": "missing"}),
    );
    assert!(err);
}

#[test]
fn context_rules_lists_protected_paths() {
    let d = valid_repo();
    let (_, text) = call(d.path(), "tempa_context", json!({"kind": "rules"}));
    assert!(
        text.contains(".env*") && text.contains("tempa.toml") && text.contains("Off-limits"),
        "{text}"
    );
}

#[test]
fn status_off_task_branch_and_on_it() {
    let d = valid_repo();
    let (err, text) = call(d.path(), "tempa_status", json!({}));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(
        !err && v["task_id"].is_null() && v["branch"] == "main",
        "{text}"
    );
    let id = task_new(d.path(), "x").unwrap();
    let (_, text) = call(d.path(), "tempa_status", json!({}));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["task_id"], id.as_str());
    assert_eq!(v["state"], "plan");
}

#[test]
fn check_returns_structured_findings() {
    let d = valid_repo();
    write(d.path(), "AGENTS.md", "# x\n");
    let (err, text) = call(d.path(), "tempa_check", json!({"contract_only": true}));
    assert!(err);
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(
        v.as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "contract.missing_heading" && f["severity"] == "error"),
        "{text}"
    );
}

// ---- Task 3: write tools ----

#[test]
fn plan_updates_given_sections_only() {
    let d = valid_repo();
    let id = task_new(d.path(), "x").unwrap();
    let path = format!("docs/plans/{id}.md");
    let before = read(d.path(), &path);
    let (err, text) = call(
        d.path(),
        "tempa_plan",
        json!({"sections": {"tujuan": "Tambah invoice.", "Risiko": "Rendah."}}),
    );
    assert!(!err, "{text}");
    let after = read(d.path(), &path);
    assert!(
        after.contains("## Goal\n\nTambah invoice.\n") && after.contains("Rendah."),
        "{after}"
    );
    assert!(text.contains("Steps"), "lists still-empty headings: {text}");
    let front = |s: &str| s.split("---").nth(1).unwrap().to_string();
    assert_eq!(front(&before), front(&after));
}

#[test]
fn plan_rejects_unknown_section_without_writing() {
    let d = valid_repo();
    let id = task_new(d.path(), "x").unwrap();
    let path = format!("docs/plans/{id}.md");
    let before = read(d.path(), &path);
    let (err, text) = call(
        d.path(),
        "tempa_plan",
        json!({"sections": {"Tujuan": "a", "Bonus": "b"}}),
    );
    assert!(
        err && text.contains("Bonus") && text.contains("Goal"),
        "{text}"
    );
    assert_eq!(read(d.path(), &path), before);
}

#[test]
fn plan_off_task_branch_is_error() {
    let d = valid_repo();
    let (err, text) = call(d.path(), "tempa_plan", json!({"sections": {"Tujuan": "a"}}));
    assert!(err && text.contains("tempa task new"), "{text}");
}

#[test]
fn plan_cannot_touch_approval() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let (err, _) = call(
        d.path(),
        "tempa_plan",
        json!({"sections": {"state": "approved"}}),
    );
    assert!(err);
}

#[test]
fn decide_requires_all_sections() {
    let d = valid_repo();
    let (err, text) = call(
        d.path(),
        "tempa_decide",
        json!({"slug": "pakai-sqlite", "sections": {"Konteks": "a"}}),
    );
    assert!(err && text.contains("Options"), "{text}");
    assert!(
        std::fs::read_dir(d.path().join("docs/decisions"))
            .unwrap()
            .all(|e| !e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("pakai-sqlite"))
    );
    let all = json!({"Konteks": "a", "Opsi": "b", "Keputusan": "c", "Konsekuensi": "d", "Agent + model": "codex / gpt-5"});
    let (err, text) = call(
        d.path(),
        "tempa_decide",
        json!({"slug": "pakai-sqlite", "sections": all}),
    );
    assert!(!err, "{text}");
    assert!(
        read(d.path(), text.trim()).contains("## Decision\n\nc\n"),
        "{text}"
    );
}

#[test]
fn log_change_validates_one_line() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let (err, text) = call(
        d.path(),
        "tempa_log_change",
        json!({"summary": "a\nb", "reason": "r", "harness": "pass", "agent": "codex / gpt-5"}),
    );
    assert!(err && text.contains("--summary"), "{text}");
    let (err, text) = call(
        d.path(),
        "tempa_log_change",
        json!({"summary": "s", "reason": "r", "harness": "pass", "agent": "codex / gpt-5"}),
    );
    assert!(!err, "{text}");
    assert!(read(d.path(), "CHANGES.md").contains("- Agent: codex / gpt-5"));
    git(d.path(), &["status", "--porcelain"]);
}

// ---- final review fixes ----

fn plan_text(p: &std::path::Path) -> String {
    let cfg = tempa_core::config::Config::load(p).unwrap();
    let id = tempa_core::task::current_task_id(p).unwrap();
    read(p, &tempa_core::task::plan_path(&cfg, &id))
}

// I3: an array/null/empty value must not erase a filled section.
#[test]
fn plan_rejects_non_string_or_empty_values() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let (err, _) = call(d.path(), "tempa_plan", json!({"sections": {"Tujuan": "a"}}));
    assert!(!err);
    let before = plan_text(d.path());
    for bad in [json!(["a", "b"]), json!(null), json!(""), json!("  ")] {
        let (err, text) = call(d.path(), "tempa_plan", json!({"sections": {"Tujuan": bad}}));
        assert!(err, "{bad}: {text}");
        assert!(text.contains("Goal"), "{text}");
    }
    assert_eq!(plan_text(d.path()), before);
}

// I3: keys that differ only by case must not slip past the all-headings check.
#[test]
fn decide_rejects_case_duplicate_keys() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let (err, text) = call(
        d.path(),
        "tempa_decide",
        json!({"slug": "dup", "sections": {
            "Konteks": "c", "konteks": "", "Opsi": "o", "Keputusan": "k",
            "Konsekuensi": "s", "Agent + model": "a"}}),
    );
    assert!(err, "{text}");
    assert!(
        !d.path()
            .join("docs/decisions")
            .read_dir()
            .unwrap()
            .any(|e| { e.unwrap().file_name().to_string_lossy().contains("dup") })
    );
}

// M3 (graded Important): a body must not inject headings that break the document.
#[test]
fn plan_rejects_headings_and_open_fences_in_bodies() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let before = plan_text(d.path());
    for bad in ["x\n## Langkah\ny", "# Title", "```\nunclosed"] {
        let (err, text) = call(d.path(), "tempa_plan", json!({"sections": {"Tujuan": bad}}));
        assert!(err, "{bad:?}: {text}");
    }
    assert_eq!(plan_text(d.path()), before);
    let ok = "x\n### Detail\n```\n## not a heading\n```";
    let (err, text) = call(d.path(), "tempa_plan", json!({"sections": {"Tujuan": ok}}));
    assert!(!err, "{text}");
    assert!(plan_text(d.path()).contains("### Detail"));
}

// I4: harness commands (minutes long) run only when asked; bad argument types are errors.
#[test]
fn check_runs_harness_only_when_asked() {
    let d = valid_repo();
    let t = read(d.path(), "tempa.toml");
    write(
        d.path(),
        "tempa.toml",
        &t.replace("[harness]", "[harness]\ntest = \"exit 3\""),
    );
    let (_, text) = call(d.path(), "tempa_check", json!({}));
    assert!(!text.contains("harness.failed"), "{text}");
    let (err, text) = call(d.path(), "tempa_check", json!({"harness": true}));
    assert!(err && text.contains("harness.failed"), "{text}");
    let (err, text) = call(d.path(), "tempa_check", json!({"harness": "yes"}));
    assert!(err && text.contains("harness"), "{text}");
}
