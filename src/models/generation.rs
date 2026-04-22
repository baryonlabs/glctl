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

    /// 이 generation이 부모 대비 바꾸려는 단일 config-knob 변경 제안.
    /// 없으면 None — 기존 YAML 호환을 위해 `#[serde(default)]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_patch: Option<ConfigPatch>,

    /// 다중 config-knob 변경 제안. 비어있으면 직렬화 시 생략 — 구버전 YAML 호환.
    /// 적용 측은 비어있지 않으면 이 Vec을 우선 사용하고, 비어있으면 단일 `config_patch`로 폴백한다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_patches: Vec<ConfigPatch>,
}

/// 단일 config 값의 변경 제안. 존재한다면 네 필드 모두 필수.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigPatch {
    /// 바꾸려는 config 키 이름 (예: "baseSpeed").
    pub key: String,
    /// 기존 값.
    pub from: f64,
    /// 제안하는 새 값.
    pub to: f64,
    /// 짧은 사람 읽기용 이유.
    pub reason: String,
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
