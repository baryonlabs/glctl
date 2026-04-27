//! `glctl fsck [--json]` — validate repository integrity.

use crate::storage;
use crate::{CliError, CliResult};
use clap::Args;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Args, Debug)]
pub struct FsckArgs {
    /// JSON object output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct FsckIssue {
    severity: &'static str,
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct FsckOutput {
    ok: bool,
    generation_count: usize,
    relation_count: usize,
    issue_count: usize,
    issues: Vec<FsckIssue>,
}

pub fn run(args: FsckArgs) -> CliResult<()> {
    let generations = storage::load_all_generations()?;
    let relations = storage::load_all_relations()?;
    let mut issues = Vec::new();

    let mut by_id: HashMap<String, usize> = HashMap::new();
    for gen in &generations {
        *by_id.entry(gen.id.clone()).or_insert(0) += 1;

        if !valid_generation_id(&gen.id) {
            issues.push(issue(
                "invalid_generation_id",
                format!("generation id '{}' does not match gen-YYYYMMDD-NNN", gen.id),
            ));
        }
        if !(0.0..=1.0).contains(&gen.metrics.score) || !gen.metrics.score.is_finite() {
            issues.push(issue(
                "invalid_score",
                format!(
                    "generation '{}' has invalid score {}",
                    gen.id, gen.metrics.score
                ),
            ));
        }
        if let Some(parent_id) = &gen.parent_id {
            if !by_id.contains_key(parent_id)
                && !generations
                    .iter()
                    .any(|candidate| candidate.id == *parent_id)
            {
                issues.push(issue(
                    "missing_parent",
                    format!(
                        "generation '{}' references missing parent '{}'",
                        gen.id, parent_id
                    ),
                ));
            }
        }
        if gen.config_patches.len() > 3 {
            issues.push(issue(
                "too_many_config_patches",
                format!(
                    "generation '{}' has {} config_patches; max is 3",
                    gen.id,
                    gen.config_patches.len()
                ),
            ));
        }
    }

    for (id, count) in &by_id {
        if *count > 1 {
            issues.push(issue(
                "duplicate_generation_id",
                format!("generation id '{}' appears {} times", id, count),
            ));
        }
    }

    let generation_ids: HashSet<&str> = generations.iter().map(|g| g.id.as_str()).collect();
    let relation_pairs: HashSet<(String, String)> = relations
        .iter()
        .map(|r| (r.from.clone(), r.to.clone()))
        .collect();

    for gen in &generations {
        if let Some(parent_id) = &gen.parent_id {
            if generation_ids.contains(parent_id.as_str())
                && !relation_pairs.contains(&(parent_id.clone(), gen.id.clone()))
            {
                issues.push(issue(
                    "missing_relation",
                    format!(
                        "generation '{}' has parent '{}' but no relation edge",
                        gen.id, parent_id
                    ),
                ));
            }
        }
    }

    for rel in &relations {
        if !generation_ids.contains(rel.from.as_str()) {
            issues.push(issue(
                "relation_missing_from",
                format!(
                    "relation {} -> {} has missing from endpoint",
                    rel.from, rel.to
                ),
            ));
        }
        if !generation_ids.contains(rel.to.as_str()) {
            issues.push(issue(
                "relation_missing_to",
                format!(
                    "relation {} -> {} has missing to endpoint",
                    rel.from, rel.to
                ),
            ));
        }
    }

    let out = FsckOutput {
        ok: issues.is_empty(),
        generation_count: generations.len(),
        relation_count: relations.len(),
        issue_count: issues.len(),
        issues,
    };

    if args.json {
        println!("{}", serde_json::to_string(&out)?);
    } else if out.ok {
        println!(
            "ok: {} generations, {} relations",
            out.generation_count, out.relation_count
        );
    } else {
        println!(
            "not ok: {} issue(s), {} generations, {} relations",
            out.issue_count, out.generation_count, out.relation_count
        );
        for issue in &out.issues {
            println!("{}: {}", issue.code, issue.message);
        }
    }

    if out.ok {
        Ok(())
    } else {
        Err(CliError::Error(format!(
            "fsck found {} issue(s)",
            out.issue_count
        )))
    }
}

fn issue(code: &'static str, message: String) -> FsckIssue {
    FsckIssue {
        severity: "error",
        code,
        message,
    }
}

fn valid_generation_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.len() != 16 {
        return false;
    }
    id.starts_with("gen-")
        && bytes[4..12].iter().all(|b| b.is_ascii_digit())
        && bytes[12] == b'-'
        && bytes[13..16].iter().all(|b| b.is_ascii_digit())
}
