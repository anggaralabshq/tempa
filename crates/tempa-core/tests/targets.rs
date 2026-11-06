use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tempa_core::config::{Agent, Config, default_toml};
use tempa_core::source::{McpServer, Skill, Source, parse_item};
use tempa_core::targets::{HEADER_MD, Rendered, header_toml, render, render_all};

fn cfg() -> Config {
    Config::parse(&default_toml("billing")).unwrap()
}

fn canon() -> Source {
    let mut mcp = BTreeMap::new();
    mcp.insert(
        "pg".to_string(),
        McpServer {
            command: Some("npx".into()),
            args: vec!["-y".into(), "x".into()],
            env: BTreeMap::from([("DB".to_string(), "${DB_URL}".to_string())]),
            url: None,
            ..Default::default()
        },
    );
    Source {
        agents_md: "# Billing API\n".into(),
        commands: vec![
            parse_item(
                "db/migrate.md".into(),
                "---\ndescription: Buat migrasi\n---\nBuat migrasi untuk $ARGUMENTS\n",
            ),
            parse_item(
                "review kode.md".into(),
                "---\ndescription: Review kode\n---\nCek kode.\n",
            ),
            parse_item(
                "review.md".into(),
                "---\ndescription: Review diff\n---\nReview perubahan: $ARGUMENTS\n",
            ),
        ],
        agents: vec![parse_item(
            "tester.md".into(),
            "---\nname: tester\ndescription: Menulis test\ntools: Read, Edit, Bash\n---\nKamu menulis test.\n",
        )],
        skills: vec![Skill {
            name: "adr".into(),
            files: vec![
                (
                    "SKILL.md".into(),
                    b"---\nname: adr\ndescription: Menulis ADR\n---\nLangkah menulis ADR.\n"
                        .to_vec(),
                ),
                ("template.md".into(), b"# ADR\n".to_vec()),
            ],
        }],
        mcp,
        claude_extra: None,
        bases: BTreeMap::new(),
    }
}

fn has(r: &Rendered, path: &str) -> bool {
    r.files.iter().any(|f| f.path == path)
}

fn text(r: &Rendered, path: &str) -> String {
    let f = r.files.iter().find(|f| f.path == path).unwrap_or_else(|| {
        panic!(
            "{path} not rendered; got {:?}",
            r.files.iter().map(|f| &f.path).collect::<Vec<_>>()
        )
    });
    String::from_utf8(f.bytes.clone()).unwrap()
}

fn json_of(r: &Rendered, path: &str) -> Value {
    serde_json::from_str(&text(r, path)).unwrap()
}

#[test]
fn claude_shim_and_commands() {
    let r = render(Agent::Claude, &cfg(), &canon());
    assert_eq!(
        text(&r, "CLAUDE.md"),
        format!("{HEADER_MD}\n\n@AGENTS.md\n")
    );
    assert_eq!(
        text(&r, ".claude/commands/db/migrate.md"),
        format!(
            "---\ndescription: Buat migrasi\n---\n{HEADER_MD}\nBuat migrasi untuk $ARGUMENTS\n"
        )
    );
    assert!(has(&r, ".claude/commands/review kode.md"));
    assert!(has(&r, ".claude/agents/tester.md"));
    assert_eq!(
        text(&r, ".claude/skills/adr/SKILL.md"),
        format!(
            "---\nname: adr\ndescription: Menulis ADR\n---\n{HEADER_MD}\nLangkah menulis ADR.\n"
        )
    );
    assert_eq!(text(&r, ".claude/skills/adr/template.md"), "# ADR\n");
    assert!(r.gaps.is_empty(), "{:?}", r.gaps);
}

#[test]
fn claude_extra_appended() {
    let mut src = canon();
    src.claude_extra = Some("Khusus\n".into());
    let r = render(Agent::Claude, &cfg(), &src);
    assert!(text(&r, "CLAUDE.md").ends_with("\n@AGENTS.md\n\nKhusus\n"));
}

#[test]
fn claude_settings_merge_deny_and_keep_hooks() {
    let mut src = canon();
    src.bases.insert(
        ".claude/settings.json".into(),
        br#"{"permissions":{"deny":["Edit(./.env*)","Bash(rm -rf:*)"]},"hooks":{"PostToolUse":[]}}"#.to_vec(),
    );
    let r = render(Agent::Claude, &cfg(), &src);
    let v = json_of(&r, ".claude/settings.json");
    assert_eq!(
        v["permissions"]["deny"],
        json!([
            "Edit(./.env*)",
            "Bash(rm -rf:*)",
            "Edit(/.env*)",
            "Edit(/tempa.toml)"
        ])
    );
    assert!(v["hooks"].is_object());
}

#[test]
fn claude_settings_without_base_still_denies() {
    let r = render(Agent::Claude, &cfg(), &canon());
    let v = json_of(&r, ".claude/settings.json");
    assert_eq!(v["permissions"]["deny"][0], "Edit(/.env*)");
}

#[test]
fn mcp_rendered_per_agent() {
    let (c, s) = (cfg(), canon());
    let rc = render(Agent::Claude, &c, &s);
    let rg = render(Agent::Gemini, &c, &s);
    let ro = render(Agent::Opencode, &c, &s);
    let rx = render(Agent::Codex, &c, &s);
    let ru = render(Agent::Cursor, &c, &s);
    assert_eq!(
        json_of(&rc, ".mcp.json")["mcpServers"]["pg"]["command"],
        "npx"
    );
    assert_eq!(
        json_of(&rg, ".gemini/settings.json")["context"]["fileName"],
        json!(["AGENTS.md"])
    );
    assert_eq!(
        json_of(&rg, ".gemini/settings.json")["mcpServers"]["pg"]["command"],
        "npx"
    );
    assert_eq!(
        json_of(&ro, "opencode.json")["mcp"]["pg"],
        json!({"type":"local","command":["npx","-y","x"],"environment":{"DB":"{env:DB_URL}"}})
    );
    let codex = text(&rx, ".codex/config.toml");
    assert!(codex.starts_with(&header_toml()), "{codex}");
    assert!(codex.contains("[mcp_servers.pg]"), "{codex}");
    let parsed: toml::Table = toml::from_str(&codex).unwrap();
    assert_eq!(parsed["mcp_servers"]["pg"]["args"][1].as_str(), Some("x"));
    assert_eq!(
        json_of(&ru, ".cursor/mcp.json")["mcpServers"]["pg"]["args"],
        json!(["-y", "x"])
    );
}

#[test]
fn remote_mcp_shapes() {
    let mut src = canon();
    src.mcp.insert(
        "web".into(),
        McpServer {
            url: Some("https://m.example".into()),
            ..Default::default()
        },
    );
    let c = cfg();
    assert_eq!(
        json_of(&render(Agent::Claude, &c, &src), ".mcp.json")["mcpServers"]["web"],
        json!({"type":"http","url":"https://m.example"})
    );
    assert_eq!(
        json_of(&render(Agent::Gemini, &c, &src), ".gemini/settings.json")["mcpServers"]["web"],
        json!({"httpUrl":"https://m.example"})
    );
    assert_eq!(
        json_of(&render(Agent::Opencode, &c, &src), "opencode.json")["mcp"]["web"],
        json!({"type":"remote","url":"https://m.example"})
    );
}

#[test]
fn no_mcp_means_no_mcp_files() {
    let mut src = canon();
    src.mcp.clear();
    let mut c = cfg();
    c.mcp.serve = false;
    assert!(!has(&render(Agent::Claude, &c, &src), ".mcp.json"));
    assert!(!has(&render(Agent::Cursor, &c, &src), ".cursor/mcp.json"));
    assert!(!has(&render(Agent::Codex, &c, &src), ".codex/config.toml"));
    // opencode.json still carries the edit deny list.
    let oc = json_of(&render(Agent::Opencode, &c, &src), "opencode.json");
    assert!(oc.get("mcp").is_none());
    let g = json_of(&render(Agent::Gemini, &c, &src), ".gemini/settings.json");
    assert!(g.get("mcpServers").is_none());
}

#[test]
fn gemini_command_toml() {
    let rg = render(Agent::Gemini, &cfg(), &canon());
    assert_eq!(
        text(&rg, ".gemini/commands/db/migrate.toml"),
        format!(
            "{}\ndescription = \"Buat migrasi\"\nprompt = \"\"\"\nBuat migrasi untuk {{{{args}}}}\n\"\"\"\n",
            header_toml()
        )
    );
}

#[test]
fn gemini_prompt_escapes_toml() {
    let mut src = canon();
    src.commands = vec![parse_item("q.md".into(), "Pakai \\n dan \"\"\" ya\n")];
    let rg = render(Agent::Gemini, &cfg(), &src);
    let parsed: toml::Table = toml::from_str(&text(&rg, ".gemini/commands/q.toml")).unwrap();
    assert_eq!(parsed["prompt"].as_str(), Some("Pakai \\n dan \"\"\" ya\n"));
    assert!(parsed.get("description").is_none());
}

#[test]
fn opencode_agent_mode_subagent_without_tools() {
    let ro = render(Agent::Opencode, &cfg(), &canon());
    let t = text(&ro, ".opencode/agents/tester.md");
    assert!(
        t.starts_with("---\ndescription: Menulis test\nmode: subagent\n---\n"),
        "{t}"
    );
    assert!(!t.contains("tools:"));
    assert_eq!(
        text(&ro, ".opencode/commands/review.md"),
        format!("---\ndescription: Review diff\n---\n{HEADER_MD}\nReview perubahan: $ARGUMENTS\n")
    );
}

#[test]
fn gemini_agent_name_description_only() {
    let rg = render(Agent::Gemini, &cfg(), &canon());
    assert_eq!(
        text(&rg, ".gemini/agents/tester.md"),
        format!(
            "---\nname: tester\ndescription: Menulis test\n---\n{HEADER_MD}\nKamu menulis test.\n"
        )
    );
}

#[test]
fn shared_skills_dir_emitted_once() {
    let all = render_all(&cfg(), &canon());
    assert_eq!(
        all.files
            .iter()
            .filter(|f| f.path == ".agents/skills/adr/SKILL.md")
            .count(),
        1
    );
    let paths: Vec<_> = all.files.iter().map(|f| f.path.clone()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn render_all_respects_enabled_agents() {
    let mut c = cfg();
    c.agents.enabled = vec![Agent::Codex];
    let all = render_all(&c, &canon());
    assert!(!all.files.iter().any(|f| f.path == "CLAUDE.md"));
    assert!(
        all.files
            .iter()
            .any(|f| f.path == ".agents/skills/adr/SKILL.md")
    );
}

#[test]
fn cursor_reads_agents_md_natively() {
    // cursor.com/docs/rules: "Cursor supports AGENTS.md in the project root and subdirectories."
    let ru = render(Agent::Cursor, &cfg(), &canon());
    assert!(!has(&ru, ".cursor/rules/tempa.mdc"));
    assert_eq!(
        text(&ru, ".cursor/commands/review.md"),
        format!("{HEADER_MD}\nReview perubahan: $ARGUMENTS\n")
    );
}

#[test]
fn cursor_mcp_env_uses_env_prefix() {
    let mut src = canon();
    src.mcp.insert(
        "web".into(),
        McpServer {
            url: Some("https://m.example/${REGION}".into()),
            headers: BTreeMap::from([("Authorization".into(), "Bearer ${TOKEN}".into())]),
            ..Default::default()
        },
    );
    let v = json_of(&render(Agent::Cursor, &cfg(), &src), ".cursor/mcp.json");
    assert_eq!(v["mcpServers"]["pg"]["env"]["DB"], "${env:DB_URL}");
    assert_eq!(
        v["mcpServers"]["web"]["url"],
        "https://m.example/${env:REGION}"
    );
    assert_eq!(
        v["mcpServers"]["web"]["headers"]["Authorization"],
        "Bearer ${env:TOKEN}"
    );
}

#[test]
fn cursor_agents_only_when_claude_is_off() {
    // Cursor also reads .claude/agents/ (cursor.com/docs/subagents): no duplicate when Claude is on.
    let mut c = cfg();
    let ru = render(Agent::Cursor, &c, &canon());
    assert!(!has(&ru, ".cursor/agents/tester.md"));
    assert!(ru.gaps.iter().all(|g| g.kind != "agents"));
    c.agents
        .enabled
        .retain(|a| !matches!(a, Agent::Claude | Agent::Codex));
    let ru = render(Agent::Cursor, &c, &canon());
    assert_eq!(
        text(&ru, ".cursor/agents/tester.md"),
        format!(
            "---\nname: tester\ndescription: Menulis test\n---\n{HEADER_MD}\nKamu menulis test.\n"
        )
    );
}

#[test]
fn cursor_cli_denies_protected_paths() {
    let mut src = canon();
    src.bases.insert(
        ".cursor/cli.json".into(),
        br#"{"permissions":{"allow":["Shell(ls)"],"deny":["Write(.env*)"]}}"#.to_vec(),
    );
    let v = json_of(&render(Agent::Cursor, &cfg(), &src), ".cursor/cli.json");
    assert_eq!(v["permissions"]["allow"], json!(["Shell(ls)"]));
    assert_eq!(
        v["permissions"]["deny"],
        json!(["Write(.env*)", "Write(tempa.toml)"])
    );
    let r = render(Agent::Cursor, &cfg(), &canon());
    assert!(r.gaps.iter().all(|g| g.kind != "settings-deny"));
}

#[test]
fn gaps_reported_never_silent() {
    let r = render(Agent::Codex, &cfg(), &canon());
    let kinds: BTreeSet<_> = r.gaps.iter().map(|g| g.kind).collect();
    assert_eq!(kinds, BTreeSet::from(["commands", "settings-deny", "mcp"]));
    assert!(has(&r, ".agents/skills/adr/SKILL.md"));
    assert_eq!(r.gaps.iter().filter(|g| g.kind == "commands").count(), 3);
    let cursor = render(Agent::Cursor, &cfg(), &canon());
    assert!(cursor.gaps.is_empty(), "{:?}", cursor.gaps);
}

#[test]
fn codex_subagents_are_toml() {
    // learn.chatgpt.com/docs/agent-configuration/subagents: name, description, developer_instructions.
    let r = render(Agent::Codex, &cfg(), &canon());
    let t: toml::Table = toml::from_str(&text(&r, ".codex/agents/tester.toml")).unwrap();
    assert_eq!(t["name"].as_str(), Some("tester"));
    assert_eq!(t["description"].as_str(), Some("Menulis test"));
    assert_eq!(
        t["developer_instructions"].as_str(),
        Some("Kamu menulis test.\n")
    );
    assert!(r.gaps.iter().all(|g| g.kind != "agents"));
}

#[test]
fn cursor_agents_skip_when_codex_writes_them() {
    let mut c = cfg();
    c.agents.enabled = vec![Agent::Codex, Agent::Cursor];
    assert!(!has(
        &render(Agent::Cursor, &c, &canon()),
        ".cursor/agents/tester.md"
    ));
}

#[test]
fn opencode_denies_protected_edits() {
    let mut src = canon();
    src.bases.insert(
        "opencode.json".into(),
        br#"{"permission":{"bash":"ask","edit":"ask"}}"#.to_vec(),
    );
    let v = json_of(&render(Agent::Opencode, &cfg(), &src), "opencode.json");
    assert_eq!(v["permission"]["bash"], "ask");
    assert_eq!(
        v["permission"]["edit"],
        json!({"*": "ask", ".env*": "deny", "tempa.toml": "deny"})
    );
    let mut bare = canon();
    bare.mcp.clear();
    let r = render(Agent::Opencode, &cfg(), &bare);
    assert_eq!(
        json_of(&r, "opencode.json")["permission"]["edit"],
        json!({"*": "allow", ".env*": "deny", "tempa.toml": "deny"})
    );
    assert!(r.gaps.iter().all(|g| g.kind != "settings-deny"));
}
