use crate::IconHandle;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn get_icon(name: String) -> Option<IconHandle> {
    let icon_name_opt = find_icon_name(&name);
    let icon_name = icon_name_opt.unwrap_or(name);

    // first check if the icon name is an absolute path to an icon file
    let icon_path = PathBuf::from(&icon_name);
    if icon_path.is_absolute() {
        return match icon_path.extension() {
            Some(ext) if ext == "png" => Some(IconHandle::Image(
                iced::widget::image::Handle::from_path(icon_path),
            )),
            Some(ext) if ext == "svg" => Some(IconHandle::Svg(
                iced::widget::svg::Handle::from_path(icon_path),
            )),
            _ => None,
        };
    }

    // try to find the icon using the icon crate
    let icons = icon::Icons::new();
    let icon_opt = icons.find_default_icon(&icon_name, 64, 1);

    if let Some(icon) = icon_opt {
        let path = icon.path();
        return match icon.file_type() {
            icon::FileType::Png => Some(IconHandle::Image(iced::widget::image::Handle::from_path(
                path,
            ))),
            icon::FileType::Svg => {
                Some(IconHandle::Svg(iced::widget::svg::Handle::from_path(path)))
            }
            icon::FileType::Xpm => None,
        };
    }

    None
}

fn find_icon_name(name: &str) -> Option<String> {
    let mut dirs = Vec::new();
    if let Some(home_dir) = dirs::home_dir().map(|p| p.to_string_lossy().into_owned()) {
        dirs.push(PathBuf::from(&format!(
            "{home_dir}/.local/share/applications"
        )));
    }
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    let entries = dirs
        .into_iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|rd| rd.filter_map(Result::ok));

    let mut ret_val = None;
    let mut found = false;

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("desktop")
            && let Ok(content) = fs::read_to_string(&path)
        {
            for line in content.lines() {
                if let Some(icon_name) = line
                    .strip_prefix("Icon=")
                    .map(|s| s.replace(['\"', '\\'], "").trim().to_string())
                    && !icon_name.is_empty()
                {
                    ret_val = Some(icon_name);
                }
                if let Some(exec_cmd) = line
                    .strip_prefix("Exec=")
                    .map(|s| s.replace(['\"', '\\'], "").trim().to_string())
                {
                    let parts: Vec<&str> = exec_cmd.split_whitespace().collect();

                    if parts.iter().any(|part| {
                        let part_name = Path::new(part)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
                        if name.len() < 15 {
                            part_name == name
                        } else {
                            part_name.contains(name)
                        }
                    }) {
                        found = true;
                    }
                }
            }
        }
        if found && ret_val.is_some() {
            return ret_val;
        }
        found = false;
        ret_val = None;
    }
    None
}
