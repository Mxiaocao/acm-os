use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub(crate) struct ResolvedNoteFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
    pub content_digest: String,
    pub windows_file_key: Option<String>,
    pub relocated: bool,
}

#[derive(Debug)]
pub(crate) enum BindingResolution {
    Ready(ResolvedNoteFile),
    LocationAnomaly,
    VaultUnavailable,
    InvalidBinding,
}

pub(crate) fn resolve_personal_note(
    active_vault: &str,
    bound_relative_path: &str,
    bound_file_key: Option<&str>,
    bound_digest: &str,
) -> BindingResolution {
    let vault = match std::fs::canonicalize(active_vault) {
        Ok(vault) if vault.is_dir() => vault,
        _ => return BindingResolution::VaultUnavailable,
    };
    let relative = Path::new(bound_relative_path);
    if !is_safe_relative_path(relative) {
        return BindingResolution::InvalidBinding;
    }

    let original = vault.join(relative);
    if let Ok(resolved) = std::fs::canonicalize(&original) {
        if resolved.starts_with(&vault) && resolved.is_file() {
            return read_resolved_note(&vault, resolved, false)
                .map(BindingResolution::Ready)
                .unwrap_or(BindingResolution::VaultUnavailable);
        }
    }

    let candidates = match markdown_files(&vault) {
        Ok(candidates) => candidates,
        Err(()) => return BindingResolution::VaultUnavailable,
    };

    if let Some(bound_file_key) = bound_file_key {
        let key_matches = candidates
            .iter()
            .filter(|path| windows_file_key(path).as_deref() == Some(bound_file_key))
            .cloned()
            .collect::<Vec<_>>();
        match key_matches.as_slice() {
            [path] => {
                return read_resolved_note(&vault, path.clone(), true)
                    .map(BindingResolution::Ready)
                    .unwrap_or(BindingResolution::VaultUnavailable);
            }
            [] => {}
            _ => return BindingResolution::LocationAnomaly,
        }
    }

    let mut digest_matches = Vec::new();
    for path in candidates {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => return BindingResolution::VaultUnavailable,
        };
        if sha256_hex(&bytes) == bound_digest {
            digest_matches.push((path, bytes));
        }
    }
    match digest_matches.as_slice() {
        [(path, bytes)] => resolved_note_from_bytes(&vault, path.clone(), bytes.clone(), true)
            .map(BindingResolution::Ready)
            .unwrap_or(BindingResolution::VaultUnavailable),
        [] => BindingResolution::LocationAnomaly,
        _ => BindingResolution::LocationAnomaly,
    }
}

pub(crate) fn resolve_relative_markdown(
    active_vault: &str,
    relative_path: &str,
) -> Result<ResolvedNoteFile, ()> {
    let vault = std::fs::canonicalize(active_vault).map_err(|_| ())?;
    if !vault.is_dir() {
        return Err(());
    }
    let relative = Path::new(relative_path);
    if !is_safe_relative_path(relative) || !is_markdown(relative) {
        return Err(());
    }
    let path = std::fs::canonicalize(vault.join(relative)).map_err(|_| ())?;
    if !path.starts_with(&vault) || !path.is_file() || !is_markdown(&path) {
        return Err(());
    }
    read_resolved_note(&vault, path, true)
}

pub(crate) fn markdown_files(vault: &Path) -> Result<Vec<PathBuf>, ()> {
    let mut files = Vec::new();
    let mut pending = vec![vault.to_path_buf()];
    let mut visited = HashSet::new();
    while let Some(directory) = pending.pop() {
        let resolved_directory = std::fs::canonicalize(&directory).map_err(|_| ())?;
        if !resolved_directory.starts_with(vault) || !visited.insert(resolved_directory.clone()) {
            continue;
        }
        for entry in std::fs::read_dir(&resolved_directory).map_err(|_| ())? {
            let entry = entry.map_err(|_| ())?;
            let file_type = entry.file_type().map_err(|_| ())?;
            if file_type.is_dir() || file_type.is_symlink() {
                let resolved = match std::fs::canonicalize(entry.path()) {
                    Ok(resolved) => resolved,
                    Err(_) => continue,
                };
                if resolved.starts_with(vault) && resolved.is_dir() {
                    pending.push(resolved);
                } else if resolved.starts_with(vault)
                    && resolved.is_file()
                    && is_markdown(&resolved)
                {
                    files.push(resolved);
                }
            } else if file_type.is_file() && is_markdown(&entry.path()) {
                let resolved = std::fs::canonicalize(entry.path()).map_err(|_| ())?;
                if resolved.starts_with(vault) {
                    files.push(resolved);
                }
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn read_resolved_note(
    vault: &Path,
    path: PathBuf,
    relocated: bool,
) -> Result<ResolvedNoteFile, ()> {
    let bytes = std::fs::read(&path).map_err(|_| ())?;
    resolved_note_from_bytes(vault, path, bytes, relocated)
}

pub(crate) fn resolved_note_from_bytes(
    vault: &Path,
    path: PathBuf,
    bytes: Vec<u8>,
    relocated: bool,
) -> Result<ResolvedNoteFile, ()> {
    let relative_path = path
        .strip_prefix(vault)
        .map_err(|_| ())?
        .to_string_lossy()
        .replace('\\', "/");
    if relative_path.is_empty() {
        return Err(());
    }
    Ok(ResolvedNoteFile {
        relative_path,
        content_digest: sha256_hex(&bytes),
        windows_file_key: windows_file_key(&path),
        bytes,
        relocated,
    })
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(windows)]
pub(crate) fn windows_file_key(path: &Path) -> Option<String> {
    use std::hash::{Hash, Hasher};

    struct FileKeyHasher(Sha256);

    impl Hasher for FileKeyHasher {
        fn finish(&self) -> u64 {
            let digest = self.0.clone().finalize();
            u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix"))
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.update(bytes);
        }
    }

    let handle = same_file::Handle::from_path(path).ok()?;
    let mut hasher = FileKeyHasher(Sha256::new());
    handle.hash(&mut hasher);
    Some(format!("same-file-1:{:016x}", hasher.finish()))
}

#[cfg(not(windows))]
pub(crate) fn windows_file_key(_path: &Path) -> Option<String> {
    None
}
