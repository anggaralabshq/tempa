//! Minimal markdown helpers: frontmatter, ATX sections, paragraphs.

pub fn normalize_eol(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Frontmatter between leading `---` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Front {
    /// Text between delimiters, without them (LF line endings).
    pub raw: String,
    /// Top-level `key: value` lines; value trimmed, outer quotes removed.
    pub pairs: Vec<(String, String)>,
}

fn is_delim(line: &str) -> bool {
    line.trim_end_matches(['\r', '\n']) == "---"
}

fn unquote(v: &str) -> &str {
    let v = v.trim();
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return &v[1..v.len() - 1];
        }
    }
    v
}

/// Split `md` into frontmatter (only when the first line is exactly `---`) and body.
pub fn split_front(md: &str) -> (Option<Front>, &str) {
    let mut lines = md.split_inclusive('\n');
    match lines.next() {
        Some(first) if is_delim(first) => {}
        _ => return (None, md),
    }
    let mut offset = md.split_inclusive('\n').next().map(str::len).unwrap_or(0);
    let mut raw = String::new();
    for line in lines {
        offset += line.len();
        if is_delim(line) {
            let pairs = raw
                .lines()
                .filter(|l| !l.starts_with([' ', '\t', '#', '-']))
                .filter_map(|l| l.split_once(':'))
                .map(|(k, v)| (k.trim().to_string(), unquote(v).to_string()))
                .collect();
            return (Some(Front { raw, pairs }), &md[offset..]);
        }
        raw.push_str(&normalize_eol(line));
    }
    (None, md)
}

/// Set `key: value` in the frontmatter, adding the frontmatter or the key if missing.
pub fn set_front(md: &str, key: &str, value: &str) -> String {
    let (front, body) = split_front(md);
    let line = format!("{key}: {value}");
    let raw = match front {
        None => format!("{line}\n"),
        Some(f) => {
            let mut found = false;
            let mut out: Vec<String> = f
                .raw
                .lines()
                .map(|l| match l.split_once(':') {
                    Some((k, _)) if !found && !l.starts_with([' ', '\t']) && k.trim() == key => {
                        found = true;
                        line.clone()
                    }
                    _ => l.to_string(),
                })
                .collect();
            if !found {
                out.push(line);
            }
            out.join("\n") + "\n"
        }
    };
    format!("---\n{raw}---\n{body}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub level: u8,
    /// Text up to the next heading of the same or higher level (sub-headings included).
    pub body: String,
}

fn heading(line: &str) -> Option<(u8, String)> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &line[hashes..];
    if !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    let title = rest.trim().trim_end_matches('#').trim().to_string();
    Some((hashes as u8, title))
}

/// ATX headings outside fenced code blocks, each with its body.
pub fn sections(md: &str) -> Vec<Section> {
    let text = normalize_eol(md);
    let lines: Vec<&str> = text.lines().collect();
    let mut heads = Vec::new();
    let mut fence = false;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
            continue;
        }
        if !fence {
            if let Some((level, title)) = heading(line) {
                heads.push((i, level, title));
            }
        }
    }
    heads
        .iter()
        .enumerate()
        .map(|(n, (i, level, title))| {
            let end = heads[n + 1..]
                .iter()
                .find(|(_, l, _)| l <= level)
                .map(|(j, _, _)| *j)
                .unwrap_or(lines.len());
            Section { title: title.clone(), level: *level, body: lines[i + 1..end].join("\n") }
        })
        .collect()
}

/// Blocks of text separated by blank lines, each trimmed.
pub fn paragraphs(md: &str) -> Vec<String> {
    let text = normalize_eol(md);
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n").trim().to_string());
                cur.clear();
            }
        } else {
            cur.push(line);
        }
    }
    if !cur.is_empty() {
        out.push(cur.join("\n").trim().to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_parses_crlf() {
        let (f, body) = split_front("---\r\ndescription: Review diff\r\n---\r\nHalo\r\n");
        assert_eq!(f.unwrap().pairs, vec![("description".to_string(), "Review diff".to_string())]);
        assert_eq!(normalize_eol(body), "Halo\n");
    }

    #[test]
    fn no_front_when_first_line_not_delimiter() {
        let (f, body) = split_front("# x\n---\na: b\n---\n");
        assert!(f.is_none());
        assert_eq!(body, "# x\n---\na: b\n---\n");
    }

    #[test]
    fn front_strips_outer_quotes() {
        let (f, _) = split_front("---\ndescription: \"Halo: dunia\"\n---\n");
        assert_eq!(f.unwrap().pairs[0].1, "Halo: dunia");
    }

    #[test]
    fn sections_ignore_code_fence() {
        let s = sections("## Stack\nRust\n```\n# bukan heading\n```\n## Perintah\n\n");
        assert_eq!(s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), ["Stack", "Perintah"]);
        assert!(s[0].body.contains("# bukan heading"));
        assert_eq!(s[1].body.trim(), "");
    }

    #[test]
    fn section_body_includes_subheadings() {
        let s = sections("## A\n### B\nx\n## C\ny\n");
        assert_eq!(s[0].title, "A");
        assert!(s[0].body.contains("### B"));
        assert_eq!(s[1].title, "B");
        assert_eq!(s[2].body.trim(), "y");
    }

    #[test]
    fn set_front_replaces_and_adds() {
        let out = set_front("---\nstate: plan\n---\nx\n", "state", "approved");
        let out = set_front(&out, "approved_by", "Ani");
        assert_eq!(out, "---\nstate: approved\napproved_by: Ani\n---\nx\n");
    }

    #[test]
    fn paragraphs_split_on_blank_lines() {
        assert_eq!(paragraphs("a\nb\n\n\nc\n"), vec!["a\nb".to_string(), "c".to_string()]);
    }
}
