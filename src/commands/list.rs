//! `glctl list [--json] [--limit N]` — 역시간순 generation 목록.

use crate::storage;
use crate::{CliError, CliResult};
use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// JSON 배열로 출력.
    #[arg(long)]
    pub json: bool,

    /// 상위 N개만.
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ListItem {
    id: String,
    parent_id: Option<String>,
    soul: String,
    score: f64,
    success: bool,
    created_at: String,
    tags: Vec<String>,
}

pub fn run(args: ListArgs) -> CliResult<()> {
    let count = storage::count_generations()?;
    if count == 0 {
        return Err(CliError::NoData("no generations found".into()));
    }

    let mut generations = storage::load_all_generations()?;
    // 역시간순
    generations.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if let Some(n) = args.limit {
        generations.truncate(n);
    }

    let items: Vec<ListItem> = generations
        .iter()
        .map(|g| ListItem {
            id: g.id.clone(),
            parent_id: g.parent_id.clone(),
            soul: g.soul.clone(),
            score: g.metrics.score,
            success: g.metrics.success,
            created_at: g.created_at.to_rfc3339(),
            tags: g.tags.clone(),
        })
        .collect();

    if args.json {
        let text = serde_json::to_string(&items)?;
        println!("{}", text);
    } else {
        println!(
            "{:<22} {:<22} {:>5} {:<8} {:<25} {}",
            "ID", "PARENT", "SCORE", "SUCCESS", "CREATED_AT", "SOUL"
        );
        for it in &items {
            let parent = it.parent_id.as_deref().unwrap_or("(seed)");
            println!(
                "{:<22} {:<22} {:>5.2} {:<8} {:<25} {}",
                it.id, parent, it.score, it.success, it.created_at, it.soul
            );
        }
    }

    Ok(())
}
