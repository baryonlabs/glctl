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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;
    use std::sync::Mutex;
    use tempfile::tempdir;

    /// Serializes tests that mutate the process-wide `GLCTL_DATA_DIR` env var.
    /// Cargo runs tests in parallel by default, so without this lock one test
    /// can clobber another's storage directory mid-run.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Round-trip: create two generations (seed + child) via the `new` command,
    /// then verify they show up through list + lineage (count, parent link, edges).
    /// Uses `GLCTL_DATA_DIR` to isolate storage in a tempdir.
    #[test]
    fn roundtrip_new_list_lineage() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempdir().expect("tempdir");
        std::env::set_var("GLCTL_DATA_DIR", tmp.path());

        // Sanity: no data yet.
        assert_eq!(count_generations().unwrap(), 0);

        // Create seed generation.
        commands::new::run(commands::new::NewArgs {
            soul: "seed".into(),
            parent: None,
            gains: vec![],
            losses: vec![],
            note: String::new(),
            score: 0.42,
            exec_time: None,
            success: true,
            tags: vec!["test".into()],
            config_patch_key: None,
            config_patch_from: None,
            config_patch_to: None,
            config_patch_reason: None,
            config_patches_json: None,
        })
        .expect("seed new");

        let seed_id = load_all_generations().unwrap()[0].id.clone();

        // Create child referencing the seed — should emit an evolved_from relation.
        commands::new::run(commands::new::NewArgs {
            soul: "child".into(),
            parent: Some(seed_id.clone()),
            gains: vec!["improved".into()],
            losses: vec![],
            note: String::new(),
            score: 0.77,
            exec_time: Some(12),
            success: false,
            tags: vec![],
            config_patch_key: None,
            config_patch_from: None,
            config_patch_to: None,
            config_patch_reason: None,
            config_patches_json: None,
        })
        .expect("child new");

        // List/lineage handlers must succeed (they print to stdout).
        commands::list::run(commands::list::ListArgs { json: true, limit: None })
            .expect("list");
        commands::lineage::run(commands::lineage::LineageArgs { json: true, from: None })
            .expect("lineage");

        // Storage-level assertions.
        assert_eq!(count_generations().unwrap(), 2);
        let gens = load_all_generations().unwrap();
        let child = gens.iter().find(|g| g.soul == "child").expect("child row");
        assert_eq!(child.parent_id.as_deref(), Some(seed_id.as_str()));
        assert!(!child.metrics.success);
        assert_eq!(child.metrics.execution_time_s, Some(12));

        let rels = load_all_relations().unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].from, seed_id);
        assert_eq!(rels[0].to, child.id);
        assert_eq!(rels[0].relation_type, crate::models::RelationType::EvolvedFrom);

        std::env::remove_var("GLCTL_DATA_DIR");
    }

    /// Round-trip a generation carrying a `config_patch`: all four fields
    /// must survive YAML serialize → deserialize via `load_generation`.
    #[test]
    fn roundtrip_generation_with_config_patch() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempdir().expect("tempdir");
        std::env::set_var("GLCTL_DATA_DIR", tmp.path());

        commands::new::run(commands::new::NewArgs {
            soul: "with-patch".into(),
            parent: None,
            gains: vec![],
            losses: vec![],
            note: String::new(),
            score: 0.5,
            exec_time: None,
            success: true,
            tags: vec![],
            config_patch_key: Some("baseSpeed".into()),
            config_patch_from: Some(1.0),
            config_patch_to: Some(1.25),
            config_patch_reason: Some("slightly faster".into()),
            config_patches_json: None,
        })
        .expect("new with config_patch");

        let id = load_all_generations().unwrap()[0].id.clone();
        let loaded = load_generation(&generation_path(&id)).expect("load");
        let cp = loaded.config_patch.expect("config_patch present");
        assert_eq!(cp.key, "baseSpeed");
        assert_eq!(cp.from, 1.0);
        assert_eq!(cp.to, 1.25);
        assert_eq!(cp.reason, "slightly faster");

        std::env::remove_var("GLCTL_DATA_DIR");
    }

    /// Round-trip a generation with a multi-knob `config_patches` Vec: both
    /// entries survive serialize → deserialize and the legacy single-patch
    /// field stays `None`.
    #[test]
    fn roundtrip_generation_with_config_patches_array() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempdir().expect("tempdir");
        std::env::set_var("GLCTL_DATA_DIR", tmp.path());

        let patches_json = r#"[
            {"key":"baseSpeed","from":1.0,"to":1.25,"reason":"faster"},
            {"key":"turnRate","from":0.4,"to":0.6,"reason":"agile"}
        ]"#;
        commands::new::run(commands::new::NewArgs {
            soul: "multi-patch".into(),
            parent: None,
            gains: vec![],
            losses: vec![],
            note: String::new(),
            score: 0.5,
            exec_time: None,
            success: true,
            tags: vec![],
            config_patch_key: None,
            config_patch_from: None,
            config_patch_to: None,
            config_patch_reason: None,
            config_patches_json: Some(patches_json.into()),
        })
        .expect("new with config_patches");

        let id = load_all_generations().unwrap()[0].id.clone();
        let loaded = load_generation(&generation_path(&id)).expect("load");
        assert!(loaded.config_patch.is_none(), "single-patch field must stay None");
        assert_eq!(loaded.config_patches.len(), 2);
        assert_eq!(loaded.config_patches[0].key, "baseSpeed");
        assert_eq!(loaded.config_patches[0].to, 1.25);
        assert_eq!(loaded.config_patches[1].key, "turnRate");
        assert_eq!(loaded.config_patches[1].reason, "agile");

        std::env::remove_var("GLCTL_DATA_DIR");
    }
}
