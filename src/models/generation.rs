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

    /// 회고 기록. 이번 generation에서 배운 운영 규칙, 만든 스킬,
    /// 잡은 버그, 영향을 준 사례를 축적한다.
    #[serde(default, skip_serializing_if = "Retrospective::is_empty")]
    pub retrospective: Retrospective,
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

/// 진화 회고. glhub의 Evolution Document가 이 필드를 중심으로 표시한다.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Retrospective {
    /// 앞으로 피해야 할 행동/패턴.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub do_not: Vec<String>,

    /// 앞으로 해야 할 행동/패턴.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub r#do: Vec<String>,

    /// 이번 generation에서 만들거나 강화한 스킬/런북/에이전트 프로필.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,

    /// 이번 generation에서 잡은 버그.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bugs_fixed: Vec<String>,

    /// 판단에 영향을 준 사례, 외부 레퍼런스, 이전 cycle 사건.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<RetrospectiveCase>,
}

impl Retrospective {
    pub fn is_empty(&self) -> bool {
        self.do_not.is_empty()
            && self.r#do.is_empty()
            && self.skills.is_empty()
            && self.bugs_fixed.is_empty()
            && self.cases.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrospectiveCase {
    /// 사례 이름 또는 짧은 제목.
    pub name: String,

    /// 이 사례가 이번 generation의 판단에 준 영향.
    pub impact: String,
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
