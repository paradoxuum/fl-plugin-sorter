use std::path::Path;

/// Determines if a path could be a VST file
pub fn is_path_vst(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        return ext == "vst3" || ext == "dll";
    }

    false
}
