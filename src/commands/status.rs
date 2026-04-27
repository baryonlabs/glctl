//! `glctl status [--json]` — summarize the company-scoped repository.

use crate::models::Generation;
use crate::storage;
use crate::CliResult;
use clap::Args;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// JSON object output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct StatusGeneration {
    id: String,
    score: f64,
    success: bool,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct StatusOutput {
    company_id: String,
    data_dir: String,
    company_dir: String,
    generation_count: usize,
    relation_count: usize,
    seed_count: usize,
    head_count: usize,
    latest: Option<StatusGeneration>,
    best: Option<StatusGeneration>,
    dangling_parent_count: usize,
}

pub fn run(args: StatusArgs) -> CliResult<()> {
    storage::ensure_dirs()?;
    let mut generations = storage::load_all_generations()?;
    let relations = storage::load_all_relations()?;

    generations.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let generation_ids: HashSet<String> = generations.iter().map(|g| g.id.clone()).collect();
    let parent_ids: HashSet<String> = generations
        .iter()
        .filter_map(|g| g.parent_id.clone())
        .collect();
    let child_ids: HashSet<String> = relations.iter().map(|r| r.to.clone()).collect();

    let out = StatusOutput {
        company_id: storage::company_id()?,
        data_dir: storage::data_dir().display().to_string(),
        company_dir: storage::company_dir()?.display().to_string(),
        generation_count: generations.len(),
        relation_count: relations.len(),
        seed_count: generations.iter().filter(|g| g.parent_id.is_none()).count(),
        head_count: generation_ids.difference(&child_ids).count(),
        latest: generations.last().map(status_generation),
        best: generations
            .iter()
            .filter(|g| g.metrics.success && g.metrics.score.is_finite())
            .max_by(|a, b| a.metrics.score.total_cmp(&b.metrics.score))
            .map(status_generation),
        dangling_parent_count: parent_ids.difference(&generation_ids).count(),
    };

    if args.json {
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!("company: {}", out.company_id);
        println!("repo: {}", out.company_dir);
        println!(
            "generations: {}  relations: {}  seeds: {}  heads: {}",
            out.generation_count, out.relation_count, out.seed_count, out.head_count
        );
        println!("dangling parents: {}", out.dangling_parent_count);
        if let Some(latest) = &out.latest {
            println!(
                "latest: {} score={:.3} success={}",
                latest.id, latest.score, latest.success
            );
        }
        if let Some(best) = &out.best {
            println!(
                "best: {} score={:.3} success={}",
                best.id, best.score, best.success
            );
        }
    }
    Ok(())
}

fn status_generation(gen: &Generation) -> StatusGeneration {
    StatusGeneration {
        id: gen.id.clone(),
        score: gen.metrics.score,
        success: gen.metrics.success,
        created_at: gen.created_at.to_rfc3339(),
    }
}
