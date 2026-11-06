mod common;
use common::write;
use tempa_core::config::{Config, default_toml};

#[test]
fn loads_canonical_tree_sorted() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(p, "tempa.toml", &default_toml("x"));
    write(p, "AGENTS.md", "# A\n");
    write(p, ".tempa/commands/z.md", "---\ndescription: Z\n---\nz\n");
    write(p, ".tempa/commands/db/migrate.md", "---\ndescription: M\n---\nm\n");
    write(p, ".tempa/skills/adr/SKILL.md", "---\nname: adr\n---\nx\n");
    write(p, ".tempa/skills/adr/template.md", "# ADR\n");
    write(p, ".tempa/mcp.toml", "[servers.pg]\ncommand=\"npx\"\n");
    write(p, ".tempa/targets/.claude/settings.json", "{}");
    write(p, ".tempa/targets/claude.md", "Khusus Claude\n");
    let cfg = Config::load(p).unwrap();
    let src = tempa_core::source::load(p, &cfg).unwrap();
    assert_eq!(src.agents_md, "# A\n");
    assert_eq!(src.commands.iter().map(|c| c.rel.as_str()).collect::<Vec<_>>(), ["db/migrate.md", "z.md"]);
    assert_eq!(tempa_core::source::front_get(&src.commands[1], "description"), Some("Z"));
    assert_eq!(src.commands[1].body, "z\n");
    assert_eq!(src.skills.len(), 1);
    assert_eq!(src.skills[0].name, "adr");
    assert_eq!(src.skills[0].files.len(), 2);
    assert_eq!(src.mcp["pg"].command.as_deref(), Some("npx"));
    assert_eq!(src.claude_extra.as_deref(), Some("Khusus Claude\n"));
    assert!(src.bases.contains_key(".claude/settings.json"));
    assert_eq!(src.bases.len(), 1);
}

#[test]
fn empty_repo_loads_empty_source() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "tempa.toml", &default_toml("x"));
    let cfg = Config::load(d.path()).unwrap();
    let src = tempa_core::source::load(d.path(), &cfg).unwrap();
    assert_eq!(src.agents_md, "");
    assert!(src.commands.is_empty() && src.skills.is_empty() && src.mcp.is_empty());
}
