use crate::IconHandle;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::{Path, PathBuf};

const TARGET_PX: u32 = 64;

/// Try to extract an icon via AppxManifest.xml (works for UWP and MSIX-packaged apps).
pub(super) fn get_icon(exe_path: &str) -> Option<IconHandle> {
    let manifest_dir = find_manifest_dir(exe_path)?;

    let icon_path = resolve_from_manifest(&manifest_dir)?;

    Some(IconHandle::Image(iced::widget::image::Handle::from_path(
        icon_path,
    )))
}

/// Walk up from the exe's directory to the filesystem root looking for AppxManifest.xml.
fn find_manifest_dir(exe_path: &str) -> Option<PathBuf> {
    let exe_path = Path::new(exe_path);

    let mut dir = exe_path.parent()?;
    loop {
        if dir.join("AppxManifest.xml").exists() {
            return Some(dir.to_path_buf());
        }
        dir = match dir.parent() {
            Some(p) => p,
            None => return None,
        };
    }
}

/// Try to resolve the icon path from the manifest content.
fn resolve_from_manifest(manifest_dir: &Path) -> Option<PathBuf> {
    let manifest_path = manifest_dir.join("AppxManifest.xml");
    let manifest = fs::read_to_string(&manifest_path).ok()?;

    let candidates = extract_logo_candidates(&manifest);

    for (logo_rel, base_size) in &candidates {
        let full = manifest_dir.join(logo_rel);
        // Try to find the variant closest to 64x64
        if let Some(path) = find_best_variant(&full, *base_size) {
            return Some(path);
        }
        // Use the exact path if no variants exist
        if full.is_file() {
            return Some(full);
        }
    }

    None
}

/// Collect logo path candidates from the manifest, with VisualElements first.
fn extract_logo_candidates(manifest: &str) -> Vec<(String, Option<u32>)> {
    let mut visual_elements = Vec::new();
    let mut package_logo: Option<String> = None;
    let mut reader = Reader::from_str(manifest);

    let mut inside_logo = false;

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e) | Event::Start(ref e)) => {
                let name = e.local_name();
                let name = name.as_ref();

                if name == b"VisualElements" {
                    for (attr, base) in [
                        (b"Square44x44Logo".as_slice(), 44u32),
                        (b"Square150x150Logo".as_slice(), 150),
                    ] {
                        if let Some(value) = get_attr(e, attr) {
                            visual_elements.push((value, Some(base)));
                        }
                    }
                } else if name == b"Logo" {
                    inside_logo = true;
                }
            }
            Ok(Event::Text(ref t)) if inside_logo => {
                if let Ok(text) = t.unescape() {
                    let logo = text.trim().to_string();
                    if !logo.is_empty() {
                        package_logo = Some(logo);
                    }
                }
                inside_logo = false;
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"Logo" => {
                inside_logo = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    if let Some(logo) = package_logo {
        visual_elements.push((logo, None));
    }

    visual_elements
}

/// Extract a UTF-8 attribute value from an XML element.
fn get_attr(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    for attr in e.attributes().filter_map(Result::ok) {
        if attr.key.as_ref() == name {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}

/// Find the icon variant whose pixel size is closest to [`TARGET_PX`].
///
/// UWP assets use two naming conventions:
/// - `Logo.targetsize-64.png` — pixel size is given directly
/// - `Logo.scale-150.png` — pixel size = base_size × scale / 100
fn find_best_variant(icon_path: &Path, base_size: Option<u32>) -> Option<PathBuf> {
    let parent = icon_path.parent()?;
    let stem = icon_path.file_stem()?.to_str()?;
    let ext = icon_path.extension()?.to_str()?;
    let suffix = format!(".{ext}");

    let entries = fs::read_dir(parent).ok()?;
    // Exclude theme/contrast qualified variants per UWP asset naming conventions:
    // contrast-standard, contrast-high, contrast-black, contrast-white,
    // theme-dark, theme-light
    let exclude = ["contrast-", "theme-"];

    let mut variants: Vec<(PathBuf, Option<u32>)> = Vec::new();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if !name.starts_with(stem) || !name.ends_with(&suffix) {
            continue;
        }
        if exclude.iter().any(|e| name.to_lowercase().contains(e)) {
            continue;
        }

        let pixel_size = extract_targetsize(&name).or_else(|| {
            let scale = extract_scale(&name)?;
            Some(base_size? * scale / 100)
        });

        variants.push((path, pixel_size));
    }

    variants
        .into_iter()
        .min_by_key(|(_, px)| px.map(|s| s.abs_diff(TARGET_PX)).unwrap_or(u32::MAX))
}

/// Extract pixel size from a `targetsize-NNN` segment in the filename.
fn extract_targetsize(name: &str) -> Option<u32> {
    let after = name.split("targetsize-").nth(1)?;
    let num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

/// Extract scale factor from a `scale-NNN` segment in the filename.
fn extract_scale(name: &str) -> Option<u32> {
    let after = name.split("scale-").nth(1)?;
    let num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}
