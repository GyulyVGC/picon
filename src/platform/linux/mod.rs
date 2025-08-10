use crate::Icon;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn get_icon_by_path(path: String) -> Option<Icon> {
    let path = Path::new(&path);
    let name = path.file_name()?.to_string_lossy().to_string();
    let bytes = load_icon_bytes(&name)?;
    Some(Icon::new(bytes))
}

fn load_icon_bytes(executable: &str) -> Option<Vec<u8>> {
    let path = find_icon_path(executable)?;
    fs::read(path).ok()
}

fn find_icon_path(executable: &str) -> Option<PathBuf> {
    let icon_dirs = [
        "/usr/share/icons/hicolor/64x64/apps",
        "~/.local/share/icons/hicolor/64x64/apps",
        "/usr/share/icons/hicolor/48x48/apps",
        "~/.local/share/icons/hicolor/48x48/apps",
        "/usr/share/icons/hicolor/42x42/apps",
        "~/.local/share/icons/hicolor/42x42/apps",
    ];

    // let extensions = ["png", "xpm", "svg"];

    for dir in &icon_dirs {
        // Expand ~ in paths
        let dir_path = shellexpand::tilde(dir).into_owned();
        // for ext in &extensions {
        let candidate = Path::new(&dir_path).join(format!("{}.png", executable));
        if candidate.exists() {
            return Some(candidate);
        }
        // }
    }
    None
}
