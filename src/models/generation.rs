//! Generation + Relation 데이터 모델.
//!
//! 스키마는 `_workspace/glctl-schema.md` v1을 따른다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// AI 에이전트의 단일 generation 실행 레코드.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    pub id: String,

    #[serde(default)]
    pub parent_id: Option<String>,

    pub created_at: DateTime<Utc>,

    pub soul: String,

    #[serde(default)]
    pub gains: Vec<String>,

    #[serde(default)]
    pub losses: Vec<String>,

    #[serde(default)]
    pub philosophical_note: String,

    pub metrics: Metrics,

    #[serde(default)]
    pub tags: Vec<String>,
}

/// 실행 품질 지표.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub score: f64,

    #[serde(default)]
    pub execution_time_s: Option<i64>,

    pub success: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            score: 0.0,
            execution_time_s: None,
            success: true,
        }
    }
}

/// Generation 간 관계 엣지.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    EvolvedFrom,
    ForkedFrom,
    MergedFrom,
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::EvolvedFrom => write!(f, "evolved_from"),
            RelationType::ForkedFrom => write!(f, "forked_from"),
            RelationType::MergedFrom => write!(f, "merged_from"),
        }
    }
}
