use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::Emitter;

pub const PERSONAL_NOTE_INVALIDATED_EVENT: &str = "personal-note-invalidated";

pub struct VaultWatcher {
    watcher: Mutex<Option<RecommendedWatcher>>,
}

impl VaultWatcher {
    pub fn new() -> Self {
        Self {
            watcher: Mutex::new(None),
        }
    }

    pub fn watch(&self, active_vault: &str, app: tauri::AppHandle) -> Result<(), ()> {
        let mut last_emit: Option<Instant> = None;
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            let Ok(event) = event else { return };
            if !event.paths.iter().any(|path| is_markdown(path)) {
                return;
            }
            let now = Instant::now();
            if last_emit.is_some_and(|previous| {
                now.duration_since(previous) < Duration::from_millis(150)
            }) {
                return;
            }
            last_emit = Some(now);
            let _ = app.emit(PERSONAL_NOTE_INVALIDATED_EVENT, ());
        })
        .map_err(|_| ())?;
        watcher
            .watch(Path::new(active_vault), RecursiveMode::Recursive)
            .map_err(|_| ())?;
        *self.watcher.lock().map_err(|_| ())? = Some(watcher);
        Ok(())
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

#[cfg(test)]
mod tests {
    use super::is_markdown;
    use notify::{RecursiveMode, Watcher};
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn invalidation_scope_only_accepts_markdown_paths() {
        assert!(is_markdown(Path::new("Problems/CF-1-A.md")));
        assert!(is_markdown(Path::new("Problems/CF-1-A.MD")));
        assert!(!is_markdown(Path::new(".obsidian/workspace.json")));
    }

    #[test]
    fn native_watcher_observes_an_external_markdown_edit() {
        let vault = tempfile::tempdir().expect("temporary vault");
        let note = vault.path().join("note.md");
        std::fs::write(&note, "before").expect("initial note");
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .expect("native watcher");
        watcher
            .watch(vault.path(), RecursiveMode::Recursive)
            .expect("watch temporary vault");

        std::fs::write(&note, "after").expect("external edit");
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut observed = false;
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(Ok(event)) if event.paths.iter().any(|path| path == &note) => {
                    observed = true;
                    break;
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        assert!(observed, "native watcher did not report the Markdown edit");
    }
}
