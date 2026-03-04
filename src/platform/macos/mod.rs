use crate::IconHandle;
use objc2_app_kit::{NSCompositingOperation, NSImage, NSWorkspace};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

pub(crate) fn get_icon(path: String) -> Option<IconHandle> {
    let path = find_app_bundle_path(&path).unwrap_or(path);

    get_icon_tiff_bytes(&path)
        .map(|b| IconHandle::Image(iced::widget::image::Handle::from_bytes(b)))
}

fn find_app_bundle_path(exe_path: &str) -> Option<String> {
    let mut current = std::path::Path::new(exe_path);
    let mut last_app_dir: Option<String> = None;

    // walk up the path (including the input)
    loop {
        if let Some(file_ext) = current.extension()
            && file_ext == "app"
        {
            last_app_dir = Some(current.to_string_lossy().to_string());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    last_app_dir
}

fn get_icon_tiff_bytes(app_path: &str) -> Option<Vec<u8>> {
    // convert Rust str to NSString
    let ns_path = NSString::from_str(app_path);

    // get shared NSWorkspace
    let ws = NSWorkspace::sharedWorkspace();

    // get icon as NSImage
    let icon = ws.iconForFile(&ns_path);

    let original_size = icon.size();
    if original_size.width == 0.0 || original_size.height == 0.0 {
        return None;
    }

    // cut 10% of the icon from each side to remove paddings
    let crop_rect = NSRect {
        origin: NSPoint {
            x: original_size.width * 0.1,
            y: original_size.height * 0.1,
        },
        size: NSSize {
            width: original_size.width * 0.8,
            height: original_size.height * 0.8,
        },
    };

    // create a drawing block that draws the icon into a 64x64 rect, cropping it to remove paddings
    let drawing_block = block2::RcBlock::new(move |dst_rect: NSRect| {
        icon.drawInRect_fromRect_operation_fraction(
            NSRect {
                origin: NSPoint {
                    x: dst_rect.origin.x,
                    y: dst_rect.origin.y,
                },
                size: NSSize {
                    width: dst_rect.size.width,
                    height: dst_rect.size.height,
                },
            },
            crop_rect,
            NSCompositingOperation::Copy,
            1.0,
        );

        true.into()
    });

    // create a new NSImage with the drawing block, which will resize and crop the icon
    let resized = NSImage::imageWithSize_flipped_drawingHandler(
        NSSize::new(64.0, 64.0),
        false,
        &drawing_block,
    );

    // get TIFF representation (NSData)
    let tiff_data = resized.TIFFRepresentation()?;

    // extract raw bytes from NSData
    Some(tiff_data.to_vec())
}
