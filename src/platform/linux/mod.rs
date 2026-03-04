use crate::IconHandle;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn get_icon(name: String) -> Option<IconHandle> {
    let icon_name_opt = find_icon_name(&name);
    let icon_name = icon_name_opt.unwrap_or(name);

    let icons: Vec<_> = linicon::lookup_icon(icon_name).with_scale(1).collect();
    for linicon in &icons {
        println!("Found icon for {icon_name}: {linicon:?}\n");
    }

    if let Some(Ok(linicon)) = icons.get(0) {
        let path = linicon.path;
        return match linicon.icon_type {
            linicon::IconType::PNG => Some(IconHandle::Image(iced::widget::image::Handle::from_path(
                path,
            ))),
            linicon::IconType::SVG => {
                Some(IconHandle::Svg(iced::widget::svg::Handle::from_path(path)))
            }
            _ => None,
        };
    }
    None
}

fn find_icon_name(name: &str) -> Option<String> {
    let mut ret_val = None;
    let mut found = false;

    let dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from(shellexpand::tilde("~/.local/share/applications").into_owned()),
    ];

    let entries = dirs
        .into_iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|rd| rd.filter_map(Result::ok));

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("desktop") {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    if let Some(icon_name) = line.strip_prefix("Icon=")
                        && !icon_name.trim().is_empty()
                    {
                        ret_val = Some(icon_name.to_string());
                    }
                    if let Some(exec_cmd) = line.strip_prefix("Exec=") {
                        if exec_cmd.contains(name) {
                            found = true;
                        }
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
