//! `glctl push` — push the current company-scoped lineage snapshot to glhub.

use crate::models::{Generation, Relation};
use crate::storage;
use crate::{CliError, CliResult};
use chrono::Utc;
use clap::Args;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Args, Debug)]
pub struct PushArgs {
    /// glhub base URL. Can also be set with GLHUB_URL.
    #[arg(long)]
    pub remote: Option<String>,
}

#[derive(Debug, Serialize)]
struct PushStatus {
    company_id: String,
    generation_count: usize,
    relation_count: usize,
    seed_count: usize,
    head_count: usize,
    latest_generation_id: Option<String>,
    best_generation_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct PushPayload {
    schema_version: &'static str,
    company_id: String,
    pushed_at: String,
    status: PushStatus,
    generations: Vec<Generation>,
    relations: Vec<Relation>,
}

pub fn run(args: PushArgs) -> CliResult<()> {
    let remote = args
        .remote
        .or_else(|| std::env::var("GLHUB_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:3201".to_string());
    let remote = remote.trim_end_matches('/');
    let company_id = storage::company_id()?;
    let mut generations = storage::load_all_generations()?;
    let relations = storage::load_all_relations()?;
    generations.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    let payload = PushPayload {
        schema_version: "glhub-push/v1",
        company_id: company_id.clone(),
        pushed_at: Utc::now().to_rfc3339(),
        status: push_status(&company_id, &generations, &relations),
        generations,
        relations,
    };

    let url = format!("{}/api/push", remote);
    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .map_err(|e| CliError::Error(format!("push failed: {}", e)))?;
    let status = response.status();
    let text = response
        .text()
        .map_err(|e| CliError::Error(format!("push response read failed: {}", e)))?;
    if !status.is_success() {
        return Err(CliError::Error(format!(
            "glhub rejected push with {}: {}",
            status, text
        )));
    }
    println!("{}", text);
    Ok(())
}

fn push_status(company_id: &str, generations: &[Generation], relations: &[Relation]) -> PushStatus {
    let generation_ids: HashSet<String> = generations.iter().map(|g| g.id.clone()).collect();
    let child_ids: HashSet<String> = relations.iter().map(|r| r.to.clone()).collect();

    PushStatus {
        company_id: company_id.to_string(),
        generation_count: generations.len(),
        relation_count: relations.len(),
        seed_count: generations.iter().filter(|g| g.parent_id.is_none()).count(),
        head_count: generation_ids.difference(&child_ids).count(),
        latest_generation_id: generations.last().map(|g| g.id.clone()),
        best_generation_id: generations
            .iter()
            .filter(|g| g.metrics.success && g.metrics.score.is_finite())
            .max_by(|a, b| a.metrics.score.total_cmp(&b.metrics.score))
            .map(|g| g.id.clone()),
    }
}
