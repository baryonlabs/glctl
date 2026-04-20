//! YAML 파일 I/O 및 데이터 디렉토리 관리.
//!
//! 레이아웃:
//!   ${GLCTL_DATA_DIR:-./data/glctl}/generations/{id}.yaml
//!   ${GLCTL_DATA_DIR:-./data/glctl}/generations/relations/{from}-{to}.yaml

use crate::models::{Generation, Relation};
use crate::{CliError, CliResult};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

/// 데이터 디렉토리 루트 결정.
pub fn data_dir() -> PathBuf {
    match std::env::var("GLCTL_DATA_DIR") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from("./data/glctl"),
    }
}

pub fn generations_dir() -> PathBuf {
    data_dir().join("generations")
}

pub fn relations_dir() -> PathBuf {
    data_dir().join("generations").join("relations")
}

pub fn generation_path(id: &str) -> PathBuf {
    generations_dir().join(format!("{}.yaml", id))
}

pub fn relation_path(from: &str, to: &str) -> PathBuf {
    relations_dir().join(format!("{}-{}.yaml", from, to))
}

/// 필요한 디렉토리를 생성한다. 이미 있으면 no-op.
pub fn ensure_dirs() -> CliResult<()> {
    fs::create_dir_all(generations_dir())?;
    fs::create_dir_all(relations_dir())?;
    Ok(())
}

/// `gen-YYYYMMDD-NNN` 형식의 새 id를 생성한다.
/// 해당 날짜에 이미 존재하는 generation 중 가장 큰 NNN+1을 사용.
pub fn next_generation_id(now: DateTime<Utc>) -> CliResult<String> {
    ensure_dirs()?;
    let date_str = now.format("%Y%m%d").to_string();
    let prefix = format!("gen-{}-", date_str);

    let mut max_seq: u32 = 0;
    let dir = generations_dir();
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.ends_with(".yaml") {
                continue;
            }
            let stem = &name[..name.len() - 5]; // strip .yaml
            if let Some(rest) = stem.strip_prefix(&prefix) {
                if let Ok(n) = rest.parse::<u32>() {
                    if n > max_seq {
                        max_seq = n;
                    }
                }
            }
        }
    }

    let next = max_seq + 1;
    Ok(format!("gen-{}-{:03}", date_str, next))
}

/// Generation 하나 저장.
pub fn save_generation(gen: &Generation) -> CliResult<()> {
    ensure_dirs()?;
    let path = generation_path(&gen.id);
    let yaml = serde_yaml::to_string(gen)?;
    fs::write(&path, yaml)?;
    Ok(())
}

/// Relation 하나 저장.
pub fn save_relation(rel: &Relation) -> CliResult<()> {
    ensure_dirs()?;
    let path = relation_path(&rel.from, &rel.to);
    let yaml = serde_yaml::to_string(rel)?;
    fs::write(&path, yaml)?;
    Ok(())
}

/// 한 파일에서 Generation 로드.
pub fn load_generation(path: &Path) -> CliResult<Generation> {
    let text = fs::read_to_string(path).map_err(|e| {
        CliError::Error(format!(
            "failed to read {}: {}",
            path.display(),
            e
        ))
    })?;
    let gen: Generation = serde_yaml::from_str(&text).map_err(|e| {
        CliError::Error(format!(
            "failed to parse {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(gen)
}

/// 한 파일에서 Relation 로드.
pub fn load_relation(path: &Path) -> CliResult<Relation> {
    let text = fs::read_to_string(path).map_err(|e| {
        CliError::Error(format!(
            "failed to read {}: {}",
            path.display(),
            e
        ))
    })?;
    let rel: Relation = serde_yaml::from_str(&text).map_err(|e| {
        CliError::Error(format!(
            "failed to parse {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(rel)
}

/// 모든 generation 로드 (정렬 보장 없음).
pub fn load_all_generations() -> CliResult<Vec<Generation>> {
    let dir = generations_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if !name.ends_with(".yaml") {
            continue;
        }
        let path = entry.path();
        out.push(load_generation(&path)?);
    }
    Ok(out)
}

/// 모든 relation 로드.
pub fn load_all_relations() -> CliResult<Vec<Relation>> {
    let dir = relations_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if !name.ends_with(".yaml") {
            continue;
        }
        let path = entry.path();
        out.push(load_relation(&path)?);
    }
    Ok(out)
}

/// Generation 개수.
pub fn count_generations() -> CliResult<usize> {
    let dir = generations_dir();
    if !dir.exists() {
        return Ok(0);
    }
    let mut n = 0;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if name.ends_with(".yaml") {
            n += 1;
        }
    }
    Ok(n)
}
