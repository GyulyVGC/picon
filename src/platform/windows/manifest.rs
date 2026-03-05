use crate::IconHandle;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const TARGET_PX: u32 = 64;

/// Try to extract an icon via AppxManifest.xml (works for UWP and MSIX-packaged apps).
pub fn get_icon(exe_path: &str) -> Option<IconHandle> {
    let path = Path::new(exe_path);
    let manifest_dir = find_manifest_dir(path)?;
    let manifest_path = manifest_dir.join("AppxManifest.xml");
    let manifest_content = fs::read_to_string(&manifest_path).ok()?;

    let icon_path = resolve_from_manifest(&manifest_dir, &manifest_content)
        .or_else(|| fuzzy_search(&manifest_dir))?;

    let bytes = fs::read(&icon_path).ok()?;
    Some(IconHandle::Image(iced::widget::image::Handle::from_bytes(
        bytes,
    )))
}

/// Walk up from the exe's directory to the filesystem root looking for AppxManifest.xml.
fn find_manifest_dir(exe_path: &Path) -> Option<PathBuf> {
    let mut dir = exe_path.parent()?;
    loop {
        if dir.join("AppxManifest.xml").exists() {
            return Some(dir.to_path_buf());
        }
        dir = match dir.parent() {
            Some(p) if p != dir => p,
            _ => return None,
        };
    }
}

/// Try to resolve the icon path from the manifest content.
fn resolve_from_manifest(package_dir: &Path, manifest: &str) -> Option<PathBuf> {
    let candidates = extract_logo_candidates(manifest);

    for (logo_rel, base_size) in &candidates {
        let full = package_dir.join(logo_rel);
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

/// Collect logo path candidates from the manifest, ordered by preference.
fn extract_logo_candidates(manifest: &str) -> Vec<(String, Option<u32>)> {
    let mut candidates = Vec::new();

    // VisualElements logos with known base sizes
    if let Some(ve_tag) = extract_visual_elements_tag(manifest) {
        for (attr, base) in [("Square44x44Logo", 44u32), ("Square150x150Logo", 150)] {
            if let Some(value) = extract_attribute(&ve_tag, attr) {
                candidates.push((value, Some(base)));
            }
        }
    }

    // Package logo (unknown base size)
    if let Some(logo) = extract_package_logo(manifest) {
        candidates.push((logo, None));
    }

    candidates
}

/// Extract the opening VisualElements tag content from the manifest.
fn extract_visual_elements_tag(manifest: &str) -> Option<String> {
    // Skip closing tags (</...VisualElements>) by looking for one NOT preceded by '/'
    let mut search_from = 0;
    loop {
        let pos = manifest[search_from..].find("VisualElements")?;
        let abs_pos = search_from + pos;
        if abs_pos > 0 && manifest.as_bytes().get(abs_pos - 1) == Some(&b'/') {
            search_from = abs_pos + "VisualElements".len();
            continue;
        }
        let section = &manifest[abs_pos..];
        let end = section.find('>')?;
        return Some(section[..end].to_string());
    }
}

/// Extract the value of an XML attribute from a tag string.
fn extract_attribute(tag: &str, attr_name: &str) -> Option<String> {
    // Ensure we match the attribute name at a word boundary (preceded by whitespace)
    let mut search_from = 0;
    loop {
        let pos = tag[search_from..].find(attr_name)?;
        let abs_pos = search_from + pos;
        // The character before the attribute name must be whitespace (or start of string)
        if abs_pos > 0 && !tag.as_bytes()[abs_pos - 1].is_ascii_whitespace() {
            search_from = abs_pos + attr_name.len();
            continue;
        }
        let after = &tag[abs_pos + attr_name.len()..];
        let after = after.strip_prefix('=')?.trim_start();
        let quote = after.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let value = &after[1..];
        let end = value.find(quote)?;
        return Some(value[..end].to_string());
    }
}

/// Extract logo path from `<Logo>...</Logo>` in the Package Properties section.
fn extract_package_logo(manifest: &str) -> Option<String> {
    let start = manifest.find("<Logo>")? + "<Logo>".len();
    let end = start + manifest[start..].find("</Logo>")?;
    Some(manifest[start..end].trim().to_string())
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

    let mut variants: Vec<(PathBuf, u32)> = Vec::new();

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

        variants.push((path, pixel_size.unwrap_or(0)));
    }

    if variants.is_empty() {
        return None;
    }

    // Prefer the variant closest to TARGET_PX; break ties by preferring >= TARGET_PX
    variants.sort_by(|(_, a), (_, b)| {
        let dist_a = a.abs_diff(TARGET_PX);
        let dist_b = b.abs_diff(TARGET_PX);
        dist_a
            .cmp(&dist_b)
            .then_with(|| (*a >= TARGET_PX).cmp(&(*b >= TARGET_PX)).reverse())
    });

    Some(variants.into_iter().next()?.0)
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

/// Fuzzy search for common icon filenames in the package folder.
fn fuzzy_search(package_dir: &Path) -> Option<PathBuf> {
    let names = ["logo", "icon", "DesktopShortcut"];
    let exts = ["png", "ico"];
    let mut candidates = Vec::new();
    collect_fuzzy_matches(package_dir, &names, &exts, &mut candidates, 0);
    candidates
        .into_iter()
        .max_by_key(|(_, size)| *size)
        .map(|(path, _)| path)
}

fn collect_fuzzy_matches(
    dir: &Path,
    names: &[&str],
    exts: &[&str],
    candidates: &mut Vec<(PathBuf, u64)>,
    depth: u32,
) {
    if depth > 5 {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_fuzzy_matches(&path, names, exts, candidates, depth + 1);
        } else if path.is_file() {
            let stem = match path.file_stem().and_then(OsStr::to_str) {
                Some(s) => s.to_lowercase(),
                None => continue,
            };
            let ext = match path.extension().and_then(OsStr::to_str) {
                Some(e) => e.to_lowercase(),
                None => continue,
            };
            let matches_name = names.iter().any(|n| stem.contains(&n.to_lowercase()));
            let matches_ext = exts.iter().any(|e| ext == *e);
            if matches_name && matches_ext {
                if let Ok(meta) = path.metadata() {
                    candidates.push((path, meta.len()));
                }
            }
        }
    }
}
