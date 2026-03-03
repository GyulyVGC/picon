use crate::IconHandle;
use objc2_app_kit::{NSCompositingOperation, NSImage, NSWorkspace};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2::AnyThread;

pub(crate) fn get_icon(path: String) -> Option<IconHandle> {
    let path = find_app_bundle_path(&path).unwrap_or(path);

    get_icon_tiff_bytes(&path)
        .map(|b| IconHandle::Image(iced::widget::image::Handle::from_bytes(b)))
}

fn find_app_bundle_path(exe_path: &str) -> Option<String> {
    let mut current = std::path::Path::new(exe_path);
    let mut last_app_dir: Option<std::path::PathBuf> = None;

    // Walk up the path, including the input
    loop {
        if let Some(file_name) = current.file_name()
            && file_name.to_string_lossy().ends_with(".app")
        {
            last_app_dir = Some(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    last_app_dir.map(|p| p.to_string_lossy().into_owned())
}

fn get_icon_tiff_bytes(app_path: &str) -> Option<Vec<u8>> {
    // Convert Rust str -> NSString
    let ns_path = NSString::from_str(app_path);

    // Get shared NSWorkspace
    let ws = NSWorkspace::sharedWorkspace();

    // Get icon as NSImage
    let icon = ws.iconForFile(&ns_path);

    let resized = NSImage::initWithSize(NSImage::alloc(), NSSize::new(64.0, 64.0));
    resized.lockFocus();

    let rect = NSRect { origin: NSPoint { x: 0.0, y: 0.0 }, size: NSSize::new(64.0, 64.0) };
    icon.drawInRect_fromRect_operation_fraction(
        rect,
        NSRect { origin: NSPoint { x: 0.0, y: 0.0 }, size: icon.size() },
        NSCompositingOperation::Copy,
        1.0,
    );
    resized.unlockFocus();

    // Get TIFF representation (NSData)
    let tiff_data = resized.TIFFRepresentation()?;

    // Extract raw bytes from NSData
    Some(tiff_data.to_vec())
}
