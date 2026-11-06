//! `tempa-app --mcp <root> --verify <file>`: the tempa MCP server injected into agent
//! sessions. Same tools as `tempa mcp`, except that a harness check is recorded.

use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::Path;

/// One JSON-RPC line in, at most one line out.
pub fn handle_line(root: &Path, verify_file: &Path, line: &str) -> Option<String> {
    let msg: Value = serde_json::from_str(line).unwrap_or(Value::Null);
    let p = &msg["params"];
    if msg["method"] == "tools/call"
        && p["name"] == "tempa_check"
        && p["arguments"]["harness"] == true
    {
        let (text, is_error) = match crate::verify::run(root, verify_file) {
            Ok(v) => (
                serde_json::to_string_pretty(&v.findings).expect("json serializes"),
                !v.ok,
            ),
            Err(e) => (e, true),
        };
        let r = json!({"jsonrpc": "2.0", "id": msg["id"],
            "result": {"content": [{"type": "text", "text": text}], "isError": is_error}});
        return Some(r.to_string());
    }
    tempa_core::mcp::handle_line(root, line)
}

/// Serve on stdin/stdout until the agent closes the pipe.
pub fn serve(root: &Path, verify_file: &Path) {
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if let Some(reply) = handle_line(root, verify_file, &line)
            && writeln!(out, "{reply}").and_then(|_| out.flush()).is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify::tests::task_repo;

    fn call(root: &std::path::Path, file: &std::path::Path, msg: Value) -> Value {
        serde_json::from_str(&handle_line(root, file, &msg.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn mcp_host_routes_harness_check_to_verify() {
        let (d, _) = task_repo("exit 0", "implement");
        let file = d.path().join("v.json");
        let r = call(
            d.path(),
            &file,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"tempa_check","arguments":{"harness":true}}}),
        );
        assert_eq!(r["result"]["isError"], false, "{r}");
        assert!(crate::verify::load(&file).unwrap().ok);
    }

    #[test]
    fn mcp_host_delegates_other_methods() {
        let (d, _) = task_repo("exit 0", "plan");
        let file = d.path().join("v.json");
        let r = call(
            d.path(),
            &file,
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        );
        assert_eq!(r["result"]["tools"].as_array().unwrap().len(), 7, "{r}");
        assert!(!file.exists(), "only a harness check records");
    }
}
