use std::sync::Mutex;

use once_cell::sync::Lazy;

/// Session-level workspace path.
static WORKSPACE_PATH: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// Set the workspace path.
pub fn set_workspace(path: String) {
    let mut ws = WORKSPACE_PATH.lock().unwrap();
    *ws = Some(path);
}

/// Get the workspace path. Returns None if not set.
pub fn get_workspace() -> Option<String> {
    WORKSPACE_PATH.lock().unwrap().clone()
}

/// Get the workspace path or error.
pub fn require_workspace() -> Result<String, String> {
    get_workspace()
        .ok_or_else(|| "No workspace set. Use clotho_set_workspace to set one.".to_string())
}

/// Try to detect a workspace by walking up from cwd.
/// Call this at server startup.
pub fn detect_and_set() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();

    loop {
        if dir.join(".clotho").is_dir() {
            let path = dir.display().to_string();
            set_workspace(path.clone());
            return Some(path);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return None,
        }
    }
}
