use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use acm_os_application::{
    KnowledgeIndexError, KnowledgeIndexProjection, KnowledgeLinkProjection,
    KnowledgeLinkResolution, KnowledgeLocationState, KnowledgeNodeProjection,
};
use sqlx::{Sqlite, Transaction};

use crate::file_binding::{markdown_files, resolved_note_from_bytes, ResolvedNoteFile};

#[derive(Debug)]
pub(crate) struct StoredKnowledgeBinding {
    pub node_id: String,
    pub relative_path: String,
    pub file_key: Option<String>,
    pub digest: String,
}

pub(crate) fn extract_wikilink_targets(markdown: &str) -> Vec<String> {
    let bytes = markdown.as_bytes();
    let mut targets = Vec::new();
    let mut cursor = 0;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'[' && bytes[cursor + 1] == b'[' {
            let start = cursor + 2;
            if let Some(end) = markdown[start..].find("]]") {
                let raw = &markdown[start..start + end];
                let target = raw
                    .split('|')
                    .next()
                    .unwrap_or_default()
                    .split('#')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .trim_end_matches(".md");
                if !target.is_empty() && !targets.iter().any(|known| known == target) {
                    targets.push(target.to_owned());
                }
                cursor = start + end + 2;
                continue;
            }
        }
        cursor += 1;
    }
    targets
}

pub(crate) fn resolve_links(
    links: Vec<(String, String, String)>,
    nodes: &[KnowledgeNodeProjection],
    non_knowledge_markdown_paths: &[String],
) -> Vec<KnowledgeLinkProjection> {
    links
        .into_iter()
        .map(|(source_kind, source_id, target_ref)| {
            let matches = nodes
                .iter()
                .filter(|node| {
                    if target_ref.contains('/') {
                        node.vault_relative_path
                            .strip_suffix(".md")
                            .is_some_and(|path| path.eq_ignore_ascii_case(&target_ref))
                    } else {
                        node.display_name.eq_ignore_ascii_case(&target_ref)
                    }
                })
                .collect::<Vec<_>>();
            let non_knowledge_matches = non_knowledge_markdown_paths
                .iter()
                .filter(|path| target_matches_path(&target_ref, path))
                .count();
            let (target_knowledge_node_id, resolution) =
                match (matches.as_slice(), non_knowledge_matches) {
                    ([node], 0) => (
                        Some(node.knowledge_node_id.clone()),
                        KnowledgeLinkResolution::Resolved,
                    ),
                    ([], 0) => (None, KnowledgeLinkResolution::Unresolved),
                    ([], _) => (None, KnowledgeLinkResolution::NonKnowledgeTarget),
                    _ => (None, KnowledgeLinkResolution::Ambiguous),
                };
            KnowledgeLinkProjection {
                source_kind,
                source_id,
                target_ref,
                target_knowledge_node_id,
                resolution,
            }
        })
        .collect()
}

fn target_matches_path(target_ref: &str, path: &str) -> bool {
    if target_ref.contains('/') {
        path.strip_suffix(".md")
            .is_some_and(|path| path.eq_ignore_ascii_case(target_ref))
    } else {
        Path::new(path)
            .file_stem()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(target_ref))
    }
}

pub(crate) fn discover_markdown(
    active_vault: &str,
    knowledge_root: &str,
) -> Result<(Vec<ResolvedNoteFile>, Vec<ResolvedNoteFile>), KnowledgeIndexError> {
    let vault = std::fs::canonicalize(active_vault)
        .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?;
    let root = std::fs::canonicalize(knowledge_root)
        .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?;
    if !vault.is_dir() || !root.is_dir() || !root.starts_with(&vault) {
        return Err(KnowledgeIndexError::KnowledgeRootUnavailable);
    }

    let in_root = read_files(
        &vault,
        markdown_files(&root).map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?,
    )?;
    let in_root_paths = in_root
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<HashSet<_>>();
    let relocation_candidates = read_files(
        &vault,
        markdown_files(&vault).map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?,
    )?
    .into_iter()
    .filter(|file| !in_root_paths.contains(&file.relative_path))
    .collect();
    Ok((in_root, relocation_candidates))
}

fn read_files(
    vault: &Path,
    paths: Vec<PathBuf>,
) -> Result<Vec<ResolvedNoteFile>, KnowledgeIndexError> {
    paths
        .into_iter()
        .map(|path| {
            let bytes =
                std::fs::read(&path).map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)?;
            resolved_note_from_bytes(vault, path, bytes, false)
                .map_err(|_| KnowledgeIndexError::KnowledgeRootUnavailable)
        })
        .collect()
}

pub(crate) async fn replace_index(
    transaction: &mut Transaction<'_, Sqlite>,
    stored: Vec<StoredKnowledgeBinding>,
    discovered: Vec<ResolvedNoteFile>,
    relocation_candidates: Vec<ResolvedNoteFile>,
) -> Result<KnowledgeIndexProjection, KnowledgeIndexError> {
    let mut unmatched = stored
        .into_iter()
        .map(|binding| (binding.node_id.clone(), binding))
        .collect::<HashMap<_, _>>();
    let mut claimed_nodes = HashSet::new();
    let mut claimed_paths = HashSet::new();
    let mut resolved = Vec::new();

    for file in &discovered {
        let node_id = match_unique_binding(&unmatched, file, &discovered, &claimed_nodes)
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        claimed_nodes.insert(node_id.clone());
        claimed_paths.insert(file.relative_path.clone());
        unmatched.remove(&node_id);
        resolved.push((node_id, clone_file(file)));
    }

    // Files outside the discovery root can only relocate an existing node. They never create one.
    for (node_id, binding) in unmatched.iter() {
        if let Some(candidate) =
            unique_relocation_match(binding, &unmatched, &relocation_candidates, &claimed_paths)
        {
            claimed_nodes.insert(node_id.clone());
            claimed_paths.insert(candidate.relative_path.clone());
            resolved.push((node_id.clone(), clone_file(candidate)));
        }
    }

    sqlx::query("DELETE FROM knowledge_discovery_index")
        .execute(&mut **transaction)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;

    let resolved_ids = resolved
        .iter()
        .map(|(node_id, _)| node_id.clone())
        .collect::<HashSet<_>>();
    sqlx::query(
        "UPDATE knowledge_file_bindings SET location_state = 'location_anomaly' \
         WHERE location_state IN ('ready', 'location_anomaly')",
    )
    .execute(&mut **transaction)
    .await
    .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
    for (node_id, _) in &resolved {
        if unmatched.contains_key(node_id) {
            sqlx::query(
                "UPDATE knowledge_file_bindings SET vault_relative_path = ?1 WHERE knowledge_node_id = ?2",
            )
            .bind(format!(".acm-os-relocation/{node_id}"))
            .bind(node_id)
            .execute(&mut **transaction)
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        }
    }

    let mut nodes = Vec::new();
    for (node_id, file) in resolved {
        sqlx::query("INSERT OR IGNORE INTO knowledge_nodes (id) VALUES (?1)")
            .bind(&node_id)
            .execute(&mut **transaction)
            .await
            .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        sqlx::query(
            "INSERT INTO knowledge_file_bindings (knowledge_node_id, vault_relative_path, windows_file_key, content_digest, location_state) \
             VALUES (?1, ?2, ?3, ?4, 'ready') ON CONFLICT(knowledge_node_id) DO UPDATE SET \
             vault_relative_path = excluded.vault_relative_path, windows_file_key = excluded.windows_file_key, \
             content_digest = excluded.content_digest, location_state = 'ready', \
             updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(&node_id)
        .bind(&file.relative_path)
        .bind(&file.windows_file_key)
        .bind(&file.content_digest)
        .execute(&mut **transaction)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        let display_name = display_name(&file.relative_path)?;
        sqlx::query(
            "INSERT INTO knowledge_discovery_index (knowledge_node_id, display_name, vault_relative_path, content_digest) \
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(&node_id)
        .bind(&display_name)
        .bind(&file.relative_path)
        .bind(&file.content_digest)
        .execute(&mut **transaction)
        .await
        .map_err(|_| KnowledgeIndexError::PersistenceUnavailable)?;
        nodes.push(KnowledgeNodeProjection {
            knowledge_node_id: node_id,
            display_name,
            vault_relative_path: file.relative_path,
            content_digest: file.content_digest,
            windows_file_key: file.windows_file_key,
            location_state: KnowledgeLocationState::Ready,
        });
    }

    let mut anomalies = Vec::new();
    for (node_id, binding) in unmatched {
        if resolved_ids.contains(&node_id) || claimed_nodes.contains(&node_id) {
            continue;
        }
        anomalies.push(KnowledgeNodeProjection {
            knowledge_node_id: node_id,
            display_name: display_name(&binding.relative_path)?,
            vault_relative_path: binding.relative_path,
            content_digest: binding.digest,
            windows_file_key: binding.file_key,
            location_state: KnowledgeLocationState::LocationAnomaly,
        });
    }
    nodes.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then_with(|| left.knowledge_node_id.cmp(&right.knowledge_node_id))
    });
    anomalies.sort_by(|left, right| left.knowledge_node_id.cmp(&right.knowledge_node_id));
    Ok(KnowledgeIndexProjection {
        nodes,
        location_anomalies: anomalies,
        identity_conflicts: Vec::new(),
    })
}

fn match_unique_binding(
    stored: &HashMap<String, StoredKnowledgeBinding>,
    file: &ResolvedNoteFile,
    discovered: &[ResolvedNoteFile],
    claimed: &HashSet<String>,
) -> Option<String> {
    let exact = stored
        .values()
        .filter(|binding| !claimed.contains(&binding.node_id))
        .filter(|binding| binding.relative_path == file.relative_path)
        .collect::<Vec<_>>();
    if exact.len() == 1 {
        return Some(exact[0].node_id.clone());
    }
    for matcher in [
        file_key_match as fn(&StoredKnowledgeBinding, &ResolvedNoteFile) -> bool,
        digest_match,
    ] {
        let matches = stored
            .values()
            .filter(|binding| !claimed.contains(&binding.node_id))
            .filter(|binding| matcher(binding, file))
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            let candidate_count = discovered
                .iter()
                .filter(|candidate| matcher(matches[0], candidate))
                .count();
            if candidate_count == 1 {
                return Some(matches[0].node_id.clone());
            }
        }
    }
    None
}

fn unique_relocation_match<'a>(
    binding: &StoredKnowledgeBinding,
    stored: &HashMap<String, StoredKnowledgeBinding>,
    candidates: &'a [ResolvedNoteFile],
    claimed_paths: &HashSet<String>,
) -> Option<&'a ResolvedNoteFile> {
    for matcher in [
        exact_path_match as fn(&StoredKnowledgeBinding, &ResolvedNoteFile) -> bool,
        file_key_match,
        digest_match,
    ] {
        let matches = candidates
            .iter()
            .filter(|candidate| !claimed_paths.contains(&candidate.relative_path))
            .filter(|candidate| matcher(binding, candidate))
            .collect::<Vec<_>>();
        if matches.len() == 1
            && stored
                .values()
                .filter(|other| matcher(other, matches[0]))
                .count()
                == 1
        {
            return matches.into_iter().next();
        }
    }
    None
}

fn exact_path_match(binding: &StoredKnowledgeBinding, file: &ResolvedNoteFile) -> bool {
    binding.relative_path == file.relative_path
}

fn file_key_match(binding: &StoredKnowledgeBinding, file: &ResolvedNoteFile) -> bool {
    binding.file_key.is_some() && binding.file_key == file.windows_file_key
}

fn digest_match(binding: &StoredKnowledgeBinding, file: &ResolvedNoteFile) -> bool {
    binding.digest == file.content_digest
}

fn clone_file(file: &ResolvedNoteFile) -> ResolvedNoteFile {
    ResolvedNoteFile {
        relative_path: file.relative_path.clone(),
        bytes: file.bytes.clone(),
        content_digest: file.content_digest.clone(),
        windows_file_key: file.windows_file_key.clone(),
        relocated: true,
    }
}

fn display_name(relative_path: &str) -> Result<String, KnowledgeIndexError> {
    Path::new(relative_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or(KnowledgeIndexError::IntegrityViolation)
}
