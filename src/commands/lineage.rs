//! `glctl lineage [--json] [--from <ID>]` — generation 계보 출력.

use crate::models::Generation;
use crate::storage;
use crate::{CliError, CliResult};
use clap::Args;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Args, Debug)]
pub struct LineageArgs {
    /// JSON으로 출력. 지정하지 않으면 사람이 읽기 쉬운 텍스트.
    #[arg(long)]
    pub json: bool,

    /// 특정 generation id에서 루트까지만 포함. 생략 시 전체.
    #[arg(long)]
    pub from: Option<String>,
}

#[derive(Debug, Serialize)]
struct LineageNode {
    id: String,
    parent_id: Option<String>,
    soul: String,
    score: f64,
    success: bool,
    created_at: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LineageEdge {
    from: String,
    to: String,
    relation_type: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct LineageOutput {
    nodes: Vec<LineageNode>,
    edges: Vec<LineageEdge>,
}

pub fn run(args: LineageArgs) -> CliResult<()> {
    let count = storage::count_generations()?;
    if count == 0 {
        return Err(CliError::NoData("no generations found".into()));
    }

    let mut generations = storage::load_all_generations()?;
    let relations = storage::load_all_relations()?;

    // 필터: --from 지정 시 해당 id에서 parent 체인만
    let generations = if let Some(leaf_id) = &args.from {
        let by_id: HashMap<String, Generation> = generations
            .iter()
            .cloned()
            .map(|g| (g.id.clone(), g))
            .collect();
        if !by_id.contains_key(leaf_id) {
            return Err(CliError::NoData(format!(
                "generation '{}' not found",
                leaf_id
            )));
        }
        let mut chain: Vec<Generation> = Vec::new();
        let mut cursor = Some(leaf_id.clone());
        let mut seen = HashSet::new();
        while let Some(id) = cursor {
            if !seen.insert(id.clone()) {
                // cycle guard
                break;
            }
            if let Some(g) = by_id.get(&id) {
                let next = g.parent_id.clone();
                chain.push(g.clone());
                cursor = next;
            } else {
                break;
            }
        }
        chain
    } else {
        generations.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        generations
    };

    let node_ids: HashSet<String> = generations.iter().map(|g| g.id.clone()).collect();

    let nodes: Vec<LineageNode> = generations
        .iter()
        .map(|g| LineageNode {
            id: g.id.clone(),
            parent_id: g.parent_id.clone(),
            soul: g.soul.clone(),
            score: g.metrics.score,
            success: g.metrics.success,
            created_at: g.created_at.to_rfc3339(),
            tags: g.tags.clone(),
        })
        .collect();

    let edges: Vec<LineageEdge> = relations
        .iter()
        .filter(|r| node_ids.contains(&r.from) && node_ids.contains(&r.to))
        .map(|r| LineageEdge {
            from: r.from.clone(),
            to: r.to.clone(),
            relation_type: r.relation_type.to_string(),
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    let out = LineageOutput { nodes, edges };

    if args.json {
        let text = serde_json::to_string(&out)?;
        println!("{}", text);
    } else {
        print_text(&out);
    }

    Ok(())
}

fn print_text(out: &LineageOutput) {
    println!(
        "Lineage ({} nodes, {} edges)",
        out.nodes.len(),
        out.edges.len()
    );
    println!();
    println!("Nodes:");
    for n in &out.nodes {
        let parent = n.parent_id.as_deref().unwrap_or("(seed)");
        println!(
            "  {:<22} parent={:<22} score={:.2} success={} soul={}",
            n.id, parent, n.score, n.success, n.soul
        );
    }
    println!();
    println!("Edges:");
    for e in &out.edges {
        println!("  {} --{}--> {}", e.from, e.relation_type, e.to);
    }
}
