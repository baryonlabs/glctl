//! `glctl graph` — Mermaid flowchart DSL 출력.

use crate::storage;
use crate::{CliError, CliResult};
use clap::Args;
use std::collections::HashSet;

#[derive(Args, Debug)]
pub struct GraphArgs {}

pub fn run(_args: GraphArgs) -> CliResult<()> {
    let count = storage::count_generations()?;
    if count == 0 {
        return Err(CliError::NoData("no generations found".into()));
    }

    let mut generations = storage::load_all_generations()?;
    generations.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let relations = storage::load_all_relations()?;

    let node_ids: HashSet<String> = generations.iter().map(|g| g.id.clone()).collect();

    let mut out = String::new();
    out.push_str("graph TD\n");

    for g in &generations {
        let safe_id = sanitize_node_id(&g.id);
        let label = format!(
            "{}<br/>{}<br/>score: {:.2}",
            escape_label(&g.id),
            escape_label(&truncate(&g.soul, 40)),
            g.metrics.score
        );
        out.push_str(&format!("  {}[\"{}\"]\n", safe_id, label));
    }

    for r in &relations {
        if !node_ids.contains(&r.from) || !node_ids.contains(&r.to) {
            continue;
        }
        let from_id = sanitize_node_id(&r.from);
        let to_id = sanitize_node_id(&r.to);
        out.push_str(&format!(
            "  {} -->|{}| {}\n",
            from_id, r.relation_type, to_id
        ));
    }

    print!("{}", out);
    Ok(())
}

/// Mermaid 노드 id로 안전하게 변환. `-`를 `_`로, 영숫자/언더스코어만 허용.
fn sanitize_node_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Mermaid 라벨 내부에서 문제를 일으키는 문자 이스케이프.
fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "&quot;")
        .replace('\n', " ")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}
