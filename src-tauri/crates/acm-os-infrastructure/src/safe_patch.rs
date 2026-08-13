use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use acm_os_application::ExtraProblemLinkTarget;

use crate::file_binding::{sha256_hex, windows_file_key};
use crate::markdown::parse_problem_markdown;

const RECOVERY_MAX_COPIES: usize = 10;
const RECOVERY_MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);
static RECOVERY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafePatchError {
    VaultUnavailable,
    BindingUnavailable,
    InvalidUtf8,
    TargetSectionMissing,
    TargetSectionAmbiguous,
    LinkAlreadyPresent,
    ConcurrentModification,
    RecoveryCopyFailed,
    WriteFailed,
    VerificationFailed,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SafePatchOutcome {
    pub relative_path: String,
    pub content_digest: String,
    pub windows_file_key: Option<String>,
}

pub(crate) fn add_extra_problem_link<F>(
    active_vault: &str,
    relative_path: &str,
    recovery_root: &Path,
    recovery_key: &str,
    target: &ExtraProblemLinkTarget,
    before_concurrency_check: F,
) -> Result<SafePatchOutcome, SafePatchError>
where
    F: FnOnce(&Path),
{
    let vault = fs::canonicalize(active_vault).map_err(|_| SafePatchError::VaultUnavailable)?;
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(SafePatchError::BindingUnavailable);
    }
    let path =
        fs::canonicalize(vault.join(relative)).map_err(|_| SafePatchError::BindingUnavailable)?;
    if !path.starts_with(&vault) || !path.is_file() {
        return Err(SafePatchError::BindingUnavailable);
    }

    let pre_bytes = fs::read(&path).map_err(|_| SafePatchError::WriteFailed)?;
    let pre_digest = sha256_hex(&pre_bytes);
    let patched = build_extra_problem_patch(&pre_bytes, target)?;
    let post_digest = sha256_hex(&patched);
    create_recovery_copy(
        recovery_root,
        recovery_key,
        &pre_bytes,
        &pre_digest,
        &post_digest,
    )?;

    before_concurrency_check(&path);
    let current = fs::read(&path).map_err(|_| SafePatchError::WriteFailed)?;
    if sha256_hex(&current) != pre_digest {
        return Err(SafePatchError::ConcurrentModification);
    }

    atomic_replace(&path, &patched)?;
    let written = fs::read(&path).map_err(|_| SafePatchError::VerificationFailed)?;
    verify_postcondition(&patched, &written, target)?;
    let relative_path = path
        .strip_prefix(&vault)
        .map_err(|_| SafePatchError::VerificationFailed)?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(SafePatchOutcome {
        relative_path,
        content_digest: sha256_hex(&written),
        windows_file_key: windows_file_key(&path),
    })
}

fn build_extra_problem_patch(
    bytes: &[u8],
    target: &ExtraProblemLinkTarget,
) -> Result<Vec<u8>, SafePatchError> {
    let markdown = std::str::from_utf8(bytes).map_err(|_| SafePatchError::InvalidUtf8)?;
    let projection = parse_problem_markdown(markdown, sha256_hex(bytes));
    let sections = projection
        .known_sections
        .iter()
        .filter(|section| section.name == "额外题目")
        .collect::<Vec<_>>();
    let section = match sections.as_slice() {
        [] => return Err(SafePatchError::TargetSectionMissing),
        [section] => *section,
        _ => return Err(SafePatchError::TargetSectionAmbiguous),
    };
    if crate::markdown::section_contains_wikilink_item(
        markdown,
        section.start_offset,
        section.end_offset,
        target.as_str(),
    ) {
        return Err(SafePatchError::LinkAlreadyPresent);
    }

    let section_bytes = &bytes[section.start_offset..section.end_offset];
    let newline: &[u8] = if section_bytes.windows(2).any(|pair| pair == b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
    };
    let mut insertion = section.end_offset;
    while insertion > section.start_offset && matches!(bytes[insertion - 1], b'\r' | b'\n') {
        insertion -= 1;
    }
    let mut patch = Vec::with_capacity(bytes.len() + target.as_str().len() + 8);
    patch.extend_from_slice(&bytes[..insertion]);
    patch.extend_from_slice(newline);
    patch.extend_from_slice(b"- [[");
    patch.extend_from_slice(target.as_str().as_bytes());
    patch.extend_from_slice(b"]]");
    patch.extend_from_slice(&bytes[insertion..]);
    Ok(patch)
}

fn contains_extra_problem_link(
    bytes: &[u8],
    target: &ExtraProblemLinkTarget,
) -> Result<bool, SafePatchError> {
    let markdown = std::str::from_utf8(bytes).map_err(|_| SafePatchError::InvalidUtf8)?;
    let projection = parse_problem_markdown(markdown, sha256_hex(bytes));
    let sections = projection
        .known_sections
        .iter()
        .filter(|section| section.name == "额外题目")
        .collect::<Vec<_>>();
    match sections.as_slice() {
        [section] => Ok(crate::markdown::section_contains_wikilink_item(
            markdown,
            section.start_offset,
            section.end_offset,
            target.as_str(),
        )),
        [] => Err(SafePatchError::TargetSectionMissing),
        _ => Err(SafePatchError::TargetSectionAmbiguous),
    }
}

fn verify_postcondition(
    expected_bytes: &[u8],
    written_bytes: &[u8],
    target: &ExtraProblemLinkTarget,
) -> Result<(), SafePatchError> {
    if written_bytes != expected_bytes || !contains_extra_problem_link(written_bytes, target)? {
        return Err(SafePatchError::VerificationFailed);
    }
    Ok(())
}

fn create_recovery_copy(
    recovery_root: &Path,
    recovery_key: &str,
    bytes: &[u8],
    pre_digest: &str,
    post_digest: &str,
) -> Result<PathBuf, SafePatchError> {
    let bucket_key = sha256_hex(recovery_key.as_bytes());
    let bucket = recovery_root.join("problem-markdown").join(bucket_key);
    fs::create_dir_all(&bucket).map_err(|_| SafePatchError::RecoveryCopyFailed)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SafePatchError::RecoveryCopyFailed)?
        .as_millis();
    let sequence = RECOVERY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = bucket.join(format!(
        "{timestamp}-{sequence}-{pre_digest}-{post_digest}.md"
    ));
    let mut copy = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| SafePatchError::RecoveryCopyFailed)?;
    copy.write_all(bytes)
        .and_then(|_| copy.sync_all())
        .map_err(|_| SafePatchError::RecoveryCopyFailed)?;
    prune_recovery_copies(&bucket, timestamp as u64)?;
    Ok(path)
}

fn prune_recovery_copies(bucket: &Path, now_millis: u64) -> Result<(), SafePatchError> {
    let max_age_millis = RECOVERY_MAX_AGE.as_millis() as u64;
    let mut copies = fs::read_dir(bucket)
        .map_err(|_| SafePatchError::RecoveryCopyFailed)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let timestamp = name.split('-').next()?.parse::<u64>().ok()?;
            Some((timestamp, entry.path()))
        })
        .collect::<Vec<_>>();
    copies.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
    for (position, (timestamp, path)) in copies.into_iter().enumerate() {
        if position >= RECOVERY_MAX_COPIES || now_millis.saturating_sub(timestamp) > max_age_millis
        {
            fs::remove_file(path).map_err(|_| SafePatchError::RecoveryCopyFailed)?;
        }
    }
    Ok(())
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), SafePatchError> {
    let parent = path.parent().ok_or(SafePatchError::WriteFailed)?;
    let permissions = fs::metadata(path)
        .map_err(|_| SafePatchError::WriteFailed)?
        .permissions();
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|_| SafePatchError::WriteFailed)?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.flush())
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|_| SafePatchError::WriteFailed)?;
    temporary
        .as_file()
        .set_permissions(permissions)
        .map_err(|_| SafePatchError::WriteFailed)?;
    temporary
        .persist(path)
        .map_err(|_| SafePatchError::WriteFailed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        add_extra_problem_link, atomic_replace, build_extra_problem_patch, prune_recovery_copies,
        verify_postcondition, SafePatchError,
    };
    use acm_os_application::ExtraProblemLinkTarget;

    #[test]
    fn patch_preserves_bom_crlf_and_all_bytes_outside_the_unique_section() {
        let before = [
            b"\xef\xbb\xbf".as_slice(),
            "# Custom\r\n\r\n## 额外题目\r\n\r\n## User section\r\nkeep me\r\n".as_bytes(),
        ]
        .concat();
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");
        let after = build_extra_problem_patch(&before, &target).expect("patch");
        let expected = [
            b"\xef\xbb\xbf".as_slice(),
            "# Custom\r\n\r\n## 额外题目\r\n- [[CF-2000-A]]\r\n\r\n## User section\r\nkeep me\r\n"
                .as_bytes(),
        ]
        .concat();
        assert_eq!(after, expected);
    }

    #[test]
    fn patch_rejects_missing_ambiguous_and_duplicate_targets() {
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");
        assert_eq!(
            build_extra_problem_patch(b"# Problem\n", &target),
            Err(SafePatchError::TargetSectionMissing)
        );
        assert_eq!(
            build_extra_problem_patch("## 额外题目\n\n## 额外题目\n".as_bytes(), &target,),
            Err(SafePatchError::TargetSectionAmbiguous)
        );
        assert_eq!(
            build_extra_problem_patch("## 额外题目\n- [[CF-2000-A]]\n".as_bytes(), &target,),
            Err(SafePatchError::LinkAlreadyPresent)
        );
    }

    #[test]
    fn recovery_retention_removes_expired_and_keeps_only_ten_recent_copies() {
        let directory = tempfile::tempdir().expect("recovery bucket");
        for timestamp in 0..12_u64 {
            std::fs::write(
                directory.path().join(format!("{timestamp}-0-digest.md")),
                b"copy",
            )
            .expect("recovery fixture");
        }
        let now = 31 * 24 * 60 * 60 * 1000_u64;
        std::fs::write(
            directory.path().join(format!("{now}-0-current.md")),
            b"current",
        )
        .expect("current fixture");
        prune_recovery_copies(directory.path(), now).expect("prune recovery copies");
        let names = std::fs::read_dir(directory.path())
            .expect("read bucket")
            .map(|entry| entry.expect("entry").file_name())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 1);
        assert!(names[0].to_string_lossy().starts_with(&now.to_string()));
    }

    #[test]
    fn recovery_retention_keeps_only_the_ten_newest_unexpired_copies() {
        let directory = tempfile::tempdir().expect("recovery bucket");
        let now = 2_000_000_000_000_u64;
        for position in 0..12_u64 {
            let timestamp = now - position;
            std::fs::write(
                directory.path().join(format!("{timestamp}-0-digest.md")),
                b"copy",
            )
            .expect("recovery fixture");
        }
        prune_recovery_copies(directory.path(), now).expect("prune recovery copies");
        assert_eq!(
            std::fs::read_dir(directory.path())
                .expect("read bucket")
                .count(),
            10
        );
    }

    #[test]
    fn transaction_creates_an_exact_recovery_copy_before_atomic_write() {
        let directory = tempfile::tempdir().expect("temporary transaction");
        let vault = directory.path().join("vault");
        let recovery = directory.path().join("recovery");
        std::fs::create_dir(&vault).expect("vault");
        let note = vault.join("note.md");
        let before = "# Problem\n\n## 额外题目\n";
        std::fs::write(&note, before).expect("note");
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");

        let outcome = add_extra_problem_link(
            vault.to_str().expect("vault path"),
            "note.md",
            &recovery,
            "codeforces:1979:A",
            &target,
            |_| {},
        )
        .expect("safe patch");

        assert_eq!(
            std::fs::read_to_string(&note).expect("patched note"),
            "# Problem\n\n## 额外题目\n- [[CF-2000-A]]\n"
        );
        assert_eq!(
            outcome.content_digest,
            crate::file_binding::sha256_hex(
                "# Problem\n\n## 额外题目\n- [[CF-2000-A]]\n".as_bytes(),
            )
        );
        let copy = walk_files(&recovery).pop().expect("recovery copy");
        assert_eq!(
            std::fs::read(copy).expect("recovery bytes"),
            before.as_bytes()
        );
    }

    #[test]
    fn transaction_cancels_when_the_note_changes_after_recovery_copy() {
        let directory = tempfile::tempdir().expect("temporary transaction");
        let vault = directory.path().join("vault");
        let recovery = directory.path().join("recovery");
        std::fs::create_dir(&vault).expect("vault");
        let note = vault.join("note.md");
        let before = "# Problem\n\n## 额外题目\n";
        std::fs::write(&note, before).expect("note");
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");

        let error = add_extra_problem_link(
            vault.to_str().expect("vault path"),
            "note.md",
            &recovery,
            "codeforces:1979:A",
            &target,
            |path| std::fs::write(path, "external edit").expect("external edit"),
        )
        .expect_err("concurrent edit must cancel");

        assert_eq!(error, SafePatchError::ConcurrentModification);
        assert_eq!(
            std::fs::read_to_string(&note).expect("current note"),
            "external edit"
        );
        let copy = walk_files(&recovery).pop().expect("recovery copy");
        assert_eq!(
            std::fs::read(copy).expect("recovery bytes"),
            before.as_bytes()
        );
    }

    #[test]
    fn recovery_bucket_stays_stable_when_the_bound_file_is_renamed() {
        let directory = tempfile::tempdir().expect("temporary transaction");
        let vault = directory.path().join("vault");
        let recovery = directory.path().join("recovery");
        std::fs::create_dir(&vault).expect("vault");
        std::fs::write(vault.join("note.md"), "## 额外题目\n").expect("note");
        let first = ExtraProblemLinkTarget::parse("CF-2000-A").expect("first target");
        add_extra_problem_link(
            vault.to_str().expect("vault path"),
            "note.md",
            &recovery,
            "codeforces:1979:A",
            &first,
            |_| {},
        )
        .expect("first patch");
        std::fs::rename(vault.join("note.md"), vault.join("renamed.md")).expect("external rename");
        let second = ExtraProblemLinkTarget::parse("CF-2000-B").expect("second target");
        add_extra_problem_link(
            vault.to_str().expect("vault path"),
            "renamed.md",
            &recovery,
            "codeforces:1979:A",
            &second,
            |_| {},
        )
        .expect("second patch");

        assert_eq!(walk_files(&recovery).len(), 2);
        assert_eq!(
            std::fs::read_dir(recovery.join("problem-markdown"))
                .expect("problem recovery root")
                .count(),
            1
        );
    }

    #[test]
    fn recovery_failure_and_path_escape_never_modify_the_note() {
        let directory = tempfile::tempdir().expect("temporary transaction");
        let vault = directory.path().join("vault");
        std::fs::create_dir(&vault).expect("vault");
        let note = vault.join("note.md");
        let before = "## 额外题目\n";
        std::fs::write(&note, before).expect("note");
        let blocked_recovery = directory.path().join("recovery-file");
        std::fs::write(&blocked_recovery, b"not a directory").expect("blocked recovery");
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");

        assert_eq!(
            add_extra_problem_link(
                vault.to_str().expect("vault path"),
                "note.md",
                &blocked_recovery,
                "codeforces:1979:A",
                &target,
                |_| {},
            ),
            Err(SafePatchError::RecoveryCopyFailed)
        );
        assert_eq!(
            std::fs::read_to_string(&note).expect("unchanged note"),
            before
        );
        assert_eq!(
            add_extra_problem_link(
                vault.to_str().expect("vault path"),
                "../recovery-file",
                directory.path(),
                "codeforces:1979:A",
                &target,
                |_| {},
            ),
            Err(SafePatchError::BindingUnavailable)
        );
    }

    #[test]
    fn write_and_verification_failures_are_explicit() {
        let directory = tempfile::tempdir().expect("temporary target");
        assert_eq!(
            atomic_replace(directory.path(), b"cannot replace a directory"),
            Err(SafePatchError::WriteFailed)
        );
        let target = ExtraProblemLinkTarget::parse("CF-2000-A").expect("target");
        assert_eq!(
            verify_postcondition(
                "## 额外题目\n- [[CF-2000-A]]\n".as_bytes(),
                "## 额外题目\ncorrupted\n".as_bytes(),
                &target,
            ),
            Err(SafePatchError::VerificationFailed)
        );
    }

    fn walk_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory).expect("recovery directory") {
                let entry = entry.expect("recovery entry");
                if entry.file_type().expect("file type").is_dir() {
                    pending.push(entry.path());
                } else {
                    files.push(entry.path());
                }
            }
        }
        files
    }
}
