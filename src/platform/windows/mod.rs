mod hicon;
mod manifest;

use crate::IconHandle;

pub(crate) fn get_icon(path: String) -> Option<IconHandle> {
    // Try manifest-based extraction first (works for UWP/MSIX packaged apps)
    if let Some(handle) = manifest::get_icon(&path) {
        return Some(handle);
    }
    // Fall back to PrivateExtractIconsW (works for regular executables)
    hicon::get_icon(&path)
}
