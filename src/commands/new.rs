//! `glctl new` — 새 generation 생성.

use crate::models::{Generation, Metrics, Relation, RelationType};
use crate::storage;
use crate::{CliError, CliResult};
use chrono::Utc;
use clap::Args;

#[derive(Args, Debug)]
pub struct NewArgs {
    /// 이번 generation의 핵심 목표 (필수).
    #[arg(long)]
    pub soul: String,

    /// 부모 generation id. 없으면 seed generation.
    #[arg(long)]
    pub parent: Option<String>,

    /// 달성한 것 (반복 지정 가능).
    #[arg(long)]
    pub gains: Vec<String>,

    /// 포기하거나 회귀한 것 (반복 지정 가능).
    #[arg(long)]
    pub losses: Vec<String>,

    /// 1줄 성찰 (philosophical_note).
    #[arg(long, default_value = "")]
    pub note: String,

    /// 품질 점수 0.0~1.0.
    #[arg(long, default_value_t = 0.0)]
    pub score: f64,

    /// 실행 소요 시간(초).
    #[arg(long)]
    pub exec_time: Option<i64>,

    /// 실행 성공 여부. 기본값 true. `--success=false`로 실패 기록 가능.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub success: bool,

    /// 태그 (반복 지정 가능).
    #[arg(long = "tag")]
    pub tags: Vec<String>,
}

pub fn run(args: NewArgs) -> CliResult<()> {
    // 검증
    if args.soul.trim().is_empty() {
        return Err(CliError::Error("--soul must not be empty".into()));
    }
    if !(0.0..=1.0).contains(&args.score) {
        return Err(CliError::Error(format!(
            "--score must be in [0.0, 1.0], got {}",
            args.score
        )));
    }

    storage::ensure_dirs()?;

    let now = Utc::now();
    let id = storage::next_generation_id(now)?;

    let gen = Generation {
        id: id.clone(),
        parent_id: args.parent.clone(),
        created_at: now,
        soul: args.soul,
        gains: args.gains,
        losses: args.losses,
        philosophical_note: args.note,
        metrics: Metrics {
            score: args.score,
            execution_time_s: args.exec_time,
            success: args.success,
        },
        tags: args.tags,
    };

    storage::save_generation(&gen)?;

    // 부모가 있으면 evolved_from relation 생성
    if let Some(parent) = &args.parent {
        let parent_path = storage::generation_path(parent);
        if !parent_path.exists() {
            // 부모가 없으면 경고하지만 실패는 아님 — relation만 생략
            eprintln!(
                "warning: parent generation '{}' not found; skipping relation",
                parent
            );
        } else {
            let rel = Relation {
                from: parent.clone(),
                to: id.clone(),
                relation_type: RelationType::EvolvedFrom,
                created_at: now,
                note: String::new(),
            };
            storage::save_relation(&rel)?;
        }
    }

    // stdout에는 생성된 id만
    println!("{}", id);
    Ok(())
}
