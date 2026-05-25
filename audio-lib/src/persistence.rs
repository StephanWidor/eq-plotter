pub fn create_from_json_file<T: serde::de::DeserializeOwned>(
    file_path: &std::path::Path,
) -> Option<T> {
    if let Ok(json) = std::fs::read_to_string(file_path) {
        if let Ok(options) = serde_json::from_str::<T>(&json) {
            return Some(options);
        }
    }
    None
}

pub fn save_to_json_file<T: serde::Serialize>(t: &T, file_path: &std::path::Path) -> bool {
    if let Some(parent_dir) = file_path.parent() {
        if std::fs::create_dir_all(&parent_dir).is_ok() {
            if let Ok(json) = serde_json::to_string_pretty(t) {
                if std::fs::write(file_path, &json).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}
