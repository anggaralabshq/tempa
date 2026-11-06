//! `tempa mcp`: a stdio MCP server (JSON-RPC 2.0, one message per line) that serves
//! the contract to any agent and validates the documents agents write.

use crate::check::{CheckOpts, check};
use crate::config::Config;
use crate::contract::{parse_changes, require_headings};
use crate::md::{set_section, split_front};
use crate::source::{self, Item, front_get};
use crate::task::{self, LogEntry};
use crate::{Severity, gitx, sync, write_atomic};
use serde_json::{Map, Value, json};
use std::path::Path;

/// Newest first; an unknown requested version gets the newest.
pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Project contract server. Call tempa_context before starting work. \
Write plans, decisions and changes entries with tempa_plan, tempa_decide and tempa_log_change; \
run tempa_check before you finish. A person approves plans (tempa task approve), never the agent.";

type ToolResult = std::result::Result<String, String>;

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// One input line → one reply line (`None` for notifications and blank lines).
pub fn handle_line(root: &Path, line: &str) -> Option<String> {
    if line.trim().is_empty() {
        return None;
    }
    let reply = match serde_json::from_str::<Value>(line) {
        Err(_) => Some(rpc_error(Value::Null, -32700, "parse error")),
        Ok(Value::Array(_)) => Some(rpc_error(
            Value::Null,
            -32600,
            "batch requests are not supported",
        )),
        Ok(msg) => handle(root, &msg),
    };
    reply.map(|v| v.to_string())
}

/// Handle one JSON-RPC message. Notifications (no `id`) get no reply.
pub fn handle(root: &Path, msg: &Value) -> Option<Value> {
    let Some(obj) = msg.as_object() else {
        return Some(rpc_error(Value::Null, -32600, "invalid request"));
    };
    let id = obj.get("id")?.clone();
    let method = obj
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = obj.get("params").cloned().unwrap_or(json!({}));
    let result = match method {
        "initialize" => {
            let asked = params["protocolVersion"].as_str().unwrap_or_default();
            let version = PROTOCOL_VERSIONS
                .iter()
                .find(|v| **v == asked)
                .unwrap_or(&PROTOCOL_VERSIONS[0]);
            json!({
                "protocolVersion": version,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "tempa", "version": env!("CARGO_PKG_VERSION")},
                "instructions": INSTRUCTIONS,
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools()}),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let (is_error, text) = match call(root, name, &args) {
                Ok(t) => (false, t),
                Err(t) => (true, t),
            };
            json!({"content": [{"type": "text", "text": text}], "isError": is_error})
        }
        _ => {
            return Some(rpc_error(
                id,
                -32601,
                &format!("method not found: {method}"),
            ));
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn schema(props: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

fn tools() -> Vec<Value> {
    let sections = json!({"type": "object", "additionalProperties": {"type": "string"},
        "description": "Heading → markdown body. Headings must be the document's required headings."});
    vec![
        json!({"name": "tempa_context", "description": "Project contract: AGENTS.md, commands, subagents, skills, rules (protected paths, harness, required headings) and the active plan. Use it to read commands or subagents your agent cannot load natively.",
            "inputSchema": schema(json!({
                "kind": {"type": "string", "enum": ["all", "commands", "agents", "skills", "rules"]},
                "name": {"type": "string", "description": "Return the full text of one command, subagent or skill"}}), &[])}),
        json!({"name": "tempa_status", "description": "Current task (from the branch), plan state, changes entries and drift, as JSON.",
            "inputSchema": schema(json!({}), &[])}),
        json!({"name": "tempa_plan", "description": "Fill or replace sections of the current task's plan. Does not approve it: a person runs `tempa task approve`.",
            "inputSchema": schema(json!({"sections": sections}), &["sections"])}),
        json!({"name": "tempa_decide", "description": "Create a decision record. Every required heading must be filled.",
            "inputSchema": schema(json!({"slug": {"type": "string", "description": "lowercase-with-hyphens"}, "sections": sections}), &["slug", "sections"])}),
        json!({"name": "tempa_log_change", "description": "Append a changes entry for the current task. `agent` names the agent and model, e.g. \"codex / gpt-5\".",
            "inputSchema": schema(json!({
                "summary": {"type": "string"}, "reason": {"type": "string"},
                "decision": {"type": "string", "description": "Path of the related decision, if any"},
                "harness": {"type": "string", "enum": ["pass", "fail", "pending"]},
                "agent": {"type": "string"}}), &["summary", "reason", "harness", "agent"])}),
        json!({"name": "tempa_check", "description": "Run tempa check (drift, contract documents, task rules) and return the findings as JSON. Set harness to true to also run the [harness] commands, which can take minutes.",
            "inputSchema": schema(json!({"harness": {"type": "boolean"}}), &[])}),
        json!({"name": "tempa_pr_body", "description": "Pull request body for the current task.",
            "inputSchema": schema(json!({}), &[])}),
    ]
}

fn call(root: &Path, name: &str, args: &Value) -> ToolResult {
    let e = |err: crate::Error| err.0;
    match name {
        "tempa_context" => context(root, args),
        "tempa_status" => status(root),
        "tempa_plan" => plan(root, args),
        "tempa_decide" => decide(root, args),
        "tempa_log_change" => log_change(root, args),
        "tempa_check" => {
            // Harness commands can take minutes; MCP clients time out tool calls.
            let harness = match &args["harness"] {
                Value::Null => false,
                Value::Bool(b) => *b,
                _ => return Err("`harness` must be true or false".into()),
            };
            let opts = CheckOpts {
                contract_only: !harness,
                base: None,
            };
            let findings = check(root, &opts).map_err(e)?;
            let list: Vec<Value> = findings
                .iter()
                .map(|f| json!({"code": f.code, "severity": match f.severity { Severity::Error => "error", Severity::Warning => "warning" }, "path": f.path, "message": f.message}))
                .collect();
            let text = serde_json::to_string_pretty(&list).expect("json serializes");
            if findings.iter().any(|f| f.severity == Severity::Error) {
                Err(text)
            } else {
                Ok(text)
            }
        }
        "tempa_pr_body" => {
            task_id(root)?;
            task::pr_body(root).map_err(e)
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

fn task_id(root: &Path) -> ToolResult {
    task::current_task_id(root).map_err(|_| {
        "not on a task branch; ask the user to run `tempa task new <slug>` first".to_string()
    })
}

fn item_name(i: &Item) -> &str {
    i.rel.strip_suffix(".md").unwrap_or(&i.rel)
}

fn context(root: &Path, args: &Value) -> ToolResult {
    let cfg = Config::load(root).map_err(|e| e.0)?;
    let src = source::load(root, &cfg).map_err(|e| e.0)?;
    let kind = args["kind"].as_str().unwrap_or("all");
    let name = args["name"].as_str();

    let items = |title: &str, list: &[Item]| -> ToolResult {
        if let Some(n) = name {
            let i = list.iter().find(|i| item_name(i) == n).ok_or_else(|| {
                let known: Vec<&str> = list.iter().map(item_name).collect();
                format!("no {title} named \"{n}\"; known: {}", known.join(", "))
            })?;
            let desc = front_get(i, "description").unwrap_or("");
            return Ok(format!("# {title}: {n}\n\n{desc}\n\n{}", i.body.trim()));
        }
        let mut s = format!("## {title}s\n\n");
        for i in list {
            s += &format!(
                "- {}: {}\n",
                item_name(i),
                front_get(i, "description").unwrap_or("-")
            );
        }
        Ok(s)
    };
    let skills = || -> ToolResult {
        let desc = |files: &[(String, Vec<u8>)]| -> (String, String) {
            let text = files
                .iter()
                .find(|(f, _)| f == "SKILL.md")
                .map(|(_, b)| String::from_utf8_lossy(b).to_string())
                .unwrap_or_default();
            let (front, body) = split_front(&text);
            let d = front
                .and_then(|f| {
                    f.pairs
                        .into_iter()
                        .find(|(k, _)| k == "description")
                        .map(|(_, v)| v)
                })
                .unwrap_or_else(|| "-".into());
            (d, body.trim().to_string())
        };
        if let Some(n) = name {
            let s = src
                .skills
                .iter()
                .find(|s| s.name == n)
                .ok_or_else(|| format!("no skill named \"{n}\""))?;
            let (d, body) = desc(&s.files);
            return Ok(format!("# skill: {n}\n\n{d}\n\n{body}"));
        }
        let mut out = "## skills\n\n".to_string();
        for s in &src.skills {
            out += &format!("- {}: {}\n", s.name, desc(&s.files).0);
        }
        Ok(out)
    };
    let rules = || {
        let mut s = "## rules\n\nProtected paths (do not edit):\n".to_string();
        for p in &cfg.gates.protected_paths {
            s += &format!("- {p}\n");
        }
        s += "\nHarness (must pass before finishing; `tempa check` runs them):\n";
        for (n, c) in &cfg.harness {
            s += &format!("- {n}: {c}\n");
        }
        let h = &cfg.headings;
        let d = &cfg.docs;
        s += "\nRequired headings:\n";
        for (doc, list) in [
            (&d.context, &h.context),
            (&d.plan, &h.plan),
            (&d.decisions, &h.decisions),
            (&d.harness, &h.harness),
        ] {
            s += &format!("- {doc}: {}\n", list.join(", "));
        }
        s += &format!(
            "\nChanges entries ({}) are written with tempa_log_change.\n",
            d.changes
        );
        s
    };

    match kind {
        "commands" => items("command", &src.commands),
        "agents" => items("subagent", &src.agents),
        "skills" => skills(),
        "rules" => Ok(rules()),
        "all" => {
            let mut s = format!(
                "{}\n\n{}\n{}\n{}\n{}",
                src.agents_md.trim(),
                items("command", &src.commands)?,
                items("subagent", &src.agents)?,
                skills()?,
                rules()
            );
            if let Ok(id) = task::current_task_id(root) {
                let path = task::plan_path(&cfg, &id);
                if let Ok(p) = std::fs::read_to_string(root.join(&path)) {
                    s += &format!("\n## active plan ({path})\n\n{}", p.trim());
                }
            }
            Ok(s)
        }
        other => Err(format!(
            "unknown kind \"{other}\"; use all, commands, agents, skills or rules"
        )),
    }
}

fn status(root: &Path) -> ToolResult {
    let cfg = Config::load(root).map_err(|e| e.0)?;
    let branch = gitx::current_branch(root).ok();
    let id = task::current_task_id(root).ok();
    let (mut plan, mut state, mut entries) = (Value::Null, Value::Null, 0);
    if let Some(id) = &id {
        let path = task::plan_path(&cfg, id);
        if let Ok(text) = std::fs::read_to_string(root.join(&path)) {
            state = split_front(&text)
                .0
                .and_then(|f| {
                    f.pairs
                        .into_iter()
                        .find(|(k, _)| k == "state")
                        .map(|(_, v)| json!(v))
                })
                .unwrap_or(Value::Null);
        }
        plan = json!(path);
        let changes = std::fs::read_to_string(root.join(&cfg.docs.changes)).unwrap_or_default();
        entries = parse_changes(&changes)
            .iter()
            .filter(|e| &e.task_id == id)
            .count();
    }
    let drift: Vec<String> = sync::drift(root)
        .map_err(|e| e.0)?
        .iter()
        .map(|d| format!("{d:?}"))
        .collect();
    let v = json!({"task_id": id, "branch": branch, "plan": plan, "state": state, "changes_entries": entries, "drift": drift});
    Ok(serde_json::to_string_pretty(&v).expect("json serializes"))
}

/// Map `sections` keys onto `allowed` headings (case-insensitive). Unknown keys → error.
fn canonical(
    sections: &Value,
    allowed: &[String],
) -> std::result::Result<Vec<(String, String)>, String> {
    let map: &Map<String, Value> = sections
        .as_object()
        .ok_or("`sections` must be an object of heading → text")?;
    let mut out = Vec::new();
    let mut unknown = Vec::new();
    for (k, v) in map {
        let Some(a) = allowed
            .iter()
            .find(|a| crate::contract::same_name(a, k.trim()))
        else {
            unknown.push(k.clone());
            continue;
        };
        if out.iter().any(|(t, _): &(String, String)| t == a) {
            return Err(format!("section {a} is given more than once"));
        }
        let body = v
            .as_str()
            .filter(|b| !b.trim().is_empty())
            .ok_or_else(|| format!("section {a} must be non-empty markdown text"))?;
        check_body(a, body)?;
        out.push((a.clone(), body.to_string()));
    }
    if !unknown.is_empty() {
        return Err(format!(
            "unknown section(s): {}; allowed: {}",
            unknown.join(", "),
            allowed.join(", ")
        ));
    }
    Ok(out)
}

/// A body must not end the section early: no `#`/`##` headings and no unclosed fence.
fn check_body(title: &str, body: &str) -> std::result::Result<(), String> {
    let mut fence: Option<&str> = None;
    for line in body.lines() {
        let t = line.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| t.starts_with(m));
        match (fence, marker) {
            (None, Some(m)) => fence = Some(m),
            (Some(f), Some(m)) if f == m => fence = None,
            (None, None) => {
                let hashes = t.chars().take_while(|c| *c == '#').count();
                let rest = &t[hashes..];
                if (1..=2).contains(&hashes) && (rest.is_empty() || rest.starts_with(' ')) {
                    return Err(format!(
                        "section {title}: line \"{line}\" is a heading; use ### or deeper inside a section"
                    ));
                }
            }
            _ => {}
        }
    }
    match fence {
        Some(f) => Err(format!("section {title}: code fence {f} is not closed")),
        None => Ok(()),
    }
}

fn apply(mut md: String, sections: &[(String, String)]) -> String {
    for (title, body) in sections {
        md = match set_section(&md, title, body) {
            Some(s) => s,
            None => format!("{}\n## {title}\n\n{}\n", md.trim_end(), body.trim()),
        };
    }
    md
}

fn plan(root: &Path, args: &Value) -> ToolResult {
    let id = task_id(root)?;
    let cfg = Config::load(root).map_err(|e| e.0)?;
    let sections = canonical(&args["sections"], &cfg.headings.plan)?;
    let path = task::plan_path(&cfg, &id);
    let text =
        std::fs::read_to_string(root.join(&path)).map_err(|_| format!("{path} not found"))?;
    let (front, body) = split_front(&text);
    let front = front
        .map(|f| format!("---\n{}---\n", f.raw))
        .unwrap_or_default();
    let out = front + &apply(body.to_string(), &sections);
    write_atomic(&root.join(&path), out.as_bytes()).map_err(|e| e.0)?;
    let empty: Vec<String> = require_headings(&out, &cfg.headings.plan)
        .into_iter()
        .map(|(h, _)| h)
        .collect();
    let rest = if empty.is_empty() {
        "all required headings filled".to_string()
    } else {
        format!("still empty: {}", empty.join(", "))
    };
    Ok(format!(
        "updated {path}; {rest}. A person approves the plan with `tempa task approve`."
    ))
}

fn decide(root: &Path, args: &Value) -> ToolResult {
    let cfg = Config::load(root).map_err(|e| e.0)?;
    let slug = args["slug"].as_str().ok_or("`slug` is required")?;
    let sections = canonical(&args["sections"], &cfg.headings.decisions)?;
    let missing: Vec<&String> = cfg
        .headings
        .decisions
        .iter()
        .filter(|h| {
            !sections
                .iter()
                .any(|(t, b)| t == *h && !b.trim().is_empty())
        })
        .collect();
    if !missing.is_empty() {
        let list: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        return Err(format!(
            "every decision heading is required; missing: {}",
            list.join(", ")
        ));
    }
    let path = task::decide(root, slug).map_err(|e| e.0)?;
    let text = std::fs::read_to_string(root.join(&path)).map_err(|e| e.to_string())?;
    write_atomic(&root.join(&path), apply(text, &sections).as_bytes()).map_err(|e| e.0)?;
    Ok(path)
}

fn log_change(root: &Path, args: &Value) -> ToolResult {
    let id = task_id(root)?;
    let s = |k: &str| args[k].as_str().unwrap_or_default().to_string();
    let entry = LogEntry {
        summary: s("summary"),
        reason: s("reason"),
        decision: args["decision"].as_str().map(String::from),
        harness: s("harness"),
        agent: s("agent"),
    };
    task::log(root, &entry).map_err(|e| e.0)?;
    Ok(format!("changes entry added for {id}"))
}
