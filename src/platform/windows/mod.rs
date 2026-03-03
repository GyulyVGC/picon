use crate::IconHandle;

pub(crate) fn get_icon_by_path(path: String) -> Option<IconHandle> {
    // let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
    // let _ = std::fs::create_dir(format!("{manifest_dir}/output"));
    let icon = windows_icons::get_icon_by_path(&path);
    if let Ok((w, h, icon)) = icon {
        // let file_name = std::path::Path::new(&path)
        //     .file_stem()
        //     .and_then(|n| n.to_str())
        //     .unwrap_or("unknown");
        println!("Successfully extracted icon for {path}");
        // icon.save(format!("{manifest_dir}/output/{file_name}.png")).unwrap();
        // return Some(Icon::new(icon.into_raw()));
        return Some(IconHandle::Image(iced::widget::image::Handle::from_rgba(
            w, h, icon,
        )));
    }
    None
}
