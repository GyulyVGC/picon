use crate::IconHandle;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn get_icon_by_path(path: String) -> Option<IconHandle> {
    let path = Path::new(&path);
    let name = path.file_stem()?.to_string_lossy().to_string();

    let icons = icon::Icons::new();
    let icon_opt = icons.find_default_icon(&name, 64, 1);

    if let Some(icon) = icon_opt {
        let path = icon.path();
        return match icon.file_type() {
            icon::FileType::Png => IconHandle::Image(iced::widget::image::Handle::from_path(path)),
            icon::FileType::Svg => IconHandle::Svg(iced::widget::svg::Handle::from_path(path)),
            _ => None,
        };
    }
    None
}
