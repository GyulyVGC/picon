use crate::Icon;


pub(crate) fn get_icon_by_path(path: String) -> Option<Icon> {
    // let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
    // let _ = std::fs::create_dir(format!("{manifest_dir}/output"));
    let icon = windows_icons::get_icon_by_path(&path);
    if let Ok(icon) = icon {
        // let file_name = std::path::Path::new(&path)
        //     .file_stem()
        //     .and_then(|n| n.to_str())
        //     .unwrap_or("unknown");
        println!("Successfully extracted icon for {path}");
        // icon.save(format!("{manifest_dir}/output/{file_name}.png")).unwrap();
        // return Some(Icon::new(icon.into_raw()));
        return Some(Icon::new(icon));
    }
    None
}
