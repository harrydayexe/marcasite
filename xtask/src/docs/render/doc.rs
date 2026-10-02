//! Rewrites rustdoc markdown for standalone markdown files.

use std::collections::HashMap;

/// Rewrites a doc comment:
///
/// - intra-doc links (keys of rustdoc's `links` map) point at the generated files via
///   `resolve`, or become plain text when the target is not documented here;
/// - hidden lines (`# ...`) are dropped from Rust code blocks, whose fences are tagged `rust`;
/// - headings are pushed down `shift` levels (capped at `######`).
pub(crate) fn rewrite(
    docs: &str,
    is_link: &dyn Fn(&str) -> bool,
    resolve: &dyn Fn(&str) -> Option<String>,
    shift: usize,
) -> String {
    // Reference definitions (`[label]: dest`) whose destination is an intra-doc link: their
    // uses become inline links (labels are per doc comment, but a page holds many) and the
    // definitions are dropped.
    let mut labels: HashMap<String, Option<String>> = HashMap::new();
    let mut in_fence = false;
    for line in docs.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        } else if !in_fence
            && let Some((label, dest)) = reference_definition(trimmed)
            && is_link(dest)
        {
            labels.insert(label.to_owned(), resolve(dest));
        }
    }

    let mut lines = Vec::new();
    let mut fence: Option<(String, bool)> = None; // (marker, is_rust)
    for line in docs.lines() {
        let trimmed = line.trim_start();
        if let Some((marker, is_rust)) = &fence {
            if trimmed.starts_with(marker.as_str())
                && trimmed
                    .trim_start_matches(marker.as_str())
                    .trim()
                    .is_empty()
            {
                fence = None;
                lines.push(line.to_owned());
            } else if *is_rust {
                let t = line.trim();
                if t == "#" || t.starts_with("# ") {
                    continue;
                }
                lines.push(match line.find("##") {
                    Some(i) if line.get(..i).is_some_and(|p| p.trim().is_empty()) => {
                        format!("{}{}", &line[..i], &line[i + 1..])
                    }
                    _ => line.to_owned(),
                });
            } else {
                lines.push(line.to_owned());
            }
            continue;
        }
        if let Some(marker) = ["```", "~~~"].iter().find(|m| trimmed.starts_with(**m)) {
            let ticks: String = trimmed
                .chars()
                .take_while(|c| *c == marker.chars().next().unwrap_or('`'))
                .collect();
            let info = trimmed[ticks.len()..].trim();
            let is_rust = is_rust_fence(info);
            let indent = &line[..line.len() - trimmed.len()];
            lines.push(if is_rust {
                format!("{indent}{ticks}rust")
            } else {
                line.to_owned()
            });
            fence = Some((ticks, is_rust));
            continue;
        }
        if reference_definition(trimmed).is_some_and(|(_, dest)| is_link(dest)) {
            continue;
        }
        let line = rewrite_links(line, is_link, resolve, &labels);
        match heading_level(line.trim_start()) {
            Some(hashes) => {
                let trimmed = line.trim_start();
                let level = (hashes + shift).min(6);
                lines.push(format!("{}{}", "#".repeat(level), &trimmed[hashes..]));
            }
            None => lines.push(line),
        }
    }
    lines.join("\n").trim().to_owned()
}

fn is_rust_fence(info: &str) -> bool {
    info.split([',', ' ']).filter(|t| !t.is_empty()).all(|t| {
        matches!(
            t,
            "rust" | "no_run" | "ignore" | "should_panic" | "compile_fail" | "test_harness"
        ) || t.starts_with("edition")
    })
}

fn heading_level(line: &str) -> Option<usize> {
    let n = line.chars().take_while(|c| *c == '#').count();
    ((1..=6).contains(&n) && line[n..].starts_with(' ')).then_some(n)
}

/// Parses `[label]: dest`.
fn reference_definition(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix('[')?;
    let end = rest.find("]:")?;
    let dest = rest[end + 2..].trim();
    (!dest.is_empty() && !dest.contains(' ')).then(|| (&rest[..end], dest))
}

/// Rewrites `[text](dest)`, `[text][label]`, `[text][]` and `[text]` links on one line,
/// leaving inline code spans alone.
fn rewrite_links(
    line: &str,
    is_link: &dyn Fn(&str) -> bool,
    resolve: &dyn Fn(&str) -> Option<String>,
    labels: &HashMap<String, Option<String>>,
) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(i) = rest.find(['[', '`']) {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if rest.starts_with('`') {
            // Copy the whole code span.
            let ticks = rest.chars().take_while(|c| *c == '`').count();
            let closing = &rest[..ticks];
            match rest[ticks..].find(closing) {
                Some(j) => {
                    out.push_str(&rest[..ticks + j + ticks]);
                    rest = &rest[ticks + j + ticks..];
                }
                None => {
                    out.push_str(closing);
                    rest = &rest[ticks..];
                }
            }
            continue;
        }
        let Some(close) = matching_bracket(rest) else {
            out.push('[');
            rest = &rest[1..];
            continue;
        };
        let text = &rest[1..close];
        let after = &rest[close + 1..];
        if let Some(inner) = after.strip_prefix('(')
            && let Some(end) = closing_paren(inner)
        {
            let dest = &inner[..end];
            let consumed = close + 1 + 1 + end + 1;
            if is_link(dest) {
                out.push_str(&linked(text, resolve(dest).as_deref()));
            } else {
                out.push_str(&rest[..consumed]);
            }
            rest = &rest[consumed..];
            continue;
        }
        if let Some(inner) = after.strip_prefix('[')
            && let Some(end) = inner.find(']')
        {
            let label = if end == 0 { text } else { &inner[..end] };
            let consumed = close + 1 + 1 + end + 1;
            if let Some(url) = labels.get(label) {
                out.push_str(&linked(text, url.as_deref()));
            } else if end == 0 && is_link(text) {
                out.push_str(&shortcut(text, resolve));
            } else {
                out.push_str(&rest[..consumed]);
            }
            rest = &rest[consumed..];
            continue;
        }
        if let Some(url) = labels.get(text) {
            out.push_str(&linked(text, url.as_deref()));
        } else if is_link(text) {
            out.push_str(&shortcut(text, resolve));
        } else {
            out.push_str(&rest[..=close]);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

fn shortcut(text: &str, resolve: &dyn Fn(&str) -> Option<String>) -> String {
    linked(text, resolve(text).as_deref())
}

/// `[text](url)`, or just `text` without a target.
fn linked(text: &str, url: Option<&str>) -> String {
    match url {
        Some(url) => format!("[{text}]({url})"),
        None => text.to_owned(),
    }
}

/// The index of the `)` closing a link destination (which may contain `()`, as in
/// `Self::next_cursor()`), given the text after the opening `(`.
fn closing_paren(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The index of the `]` closing the `[` at the start of `s`, skipping code spans.
fn matching_bracket(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_code = false;
    for (i, c) in s.char_indices() {
        match c {
            '`' => in_code = !in_code,
            '[' if !in_code => depth += 1,
            ']' if !in_code => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::rewrite;

    fn run(docs: &str) -> String {
        let is_link = |k: &str| {
            matches!(
                k,
                "`Foo`" | "Foo::bar" | "Foo::bar()" | "crate::Gone" | "`Gone`"
            )
        };
        let resolve = |k: &str| match k {
            "`Foo`" => Some("a.md#struct.Foo".to_owned()),
            "Foo::bar" | "Foo::bar()" => Some("a.md#Foo.fn.bar".to_owned()),
            _ => None,
        };
        rewrite(docs, &is_link, &resolve, 2)
    }

    #[test]
    fn rewrites_links() {
        assert_eq!(
            run("See [`Foo`] and [x](Foo::bar)."),
            "See [`Foo`](a.md#struct.Foo) and [x](a.md#Foo.fn.bar)."
        );
        assert_eq!(
            run("See [`Gone`] and [y](crate::Gone)."),
            "See `Gone` and y."
        );
        assert_eq!(
            run("Call [`bar()`](Foo::bar()) (once)."),
            "Call [`bar()`](a.md#Foo.fn.bar) (once)."
        );
        assert_eq!(
            run("Keep [ext](https://x.y) and `[a]`."),
            "Keep [ext](https://x.y) and `[a]`."
        );
        assert_eq!(run("Use [g][l].\n\n[l]: crate::Gone"), "Use g.");
        assert_eq!(
            run("Use [h][m] and [m].\n\n[m]: Foo::bar"),
            "Use [h](a.md#Foo.fn.bar) and [m](a.md#Foo.fn.bar)."
        );
    }

    #[test]
    fn code_blocks_and_headings() {
        assert_eq!(
            run("# Errors\n```no_run\n# hidden\nshown();\n```"),
            "### Errors\n```rust\nshown();\n```"
        );
        assert_eq!(run("```json\n# kept\n```"), "```json\n# kept\n```");
    }
}
