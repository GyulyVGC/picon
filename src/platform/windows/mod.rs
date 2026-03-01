use crate::Icon;
use std::fmt::format;
// use winapi::um::wingdi::{ CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, RGBQUAD };
// use winapi::um::shellapi::{ SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON };
// use winapi::um::winbase::{ GlobalAlloc, GlobalLock, GHND, GlobalUnlock, GlobalFree };
// use winapi::um::winuser::{ GetIconInfo, ICONINFO, DestroyIcon };
// use winapi::shared::minwindef::DWORD;
// use iced::widget::image::Handle;
// use widestring::U16CString;
// use std::{mem, ptr, slice};

pub(crate) fn get_icon_by_path(path: String) -> Option<Icon> {
    let _ = std::fs::create_dir("output");
    let icon = windows_icons::get_icon_by_path(&path);
    if let Ok(icon) = icon {
        println!("Successfully extracted icon for {path}");
        icon.save("output/icon.png").unwrap();
        return Some(Icon::new(icon.into_raw()));
    }
    None
}

// pub fn extract_icon_as_handle(path: &str) -> Result<Handle, Box<dyn std::error::Error>> {
//     unsafe {
//         let mut shfi = SHFILEINFOW {
//             hIcon: ptr::null_mut(), iIcon: 0,
//             dwAttributes: 0,
//             szDisplayName: [0; 260],
//             szTypeName: [0; 80],
//         };
//
//         SHGetFileInfoW(
//             U16CString::from_str(path)?.as_ptr(),
//             0,
//             &mut shfi,
//             mem::size_of::<SHFILEINFOW>() as DWORD,
//             SHGFI_ICON | SHGFI_LARGEICON,
//         );
//
//         if shfi.hIcon.is_null() {
//             return Err("No icon found.".into());
//         }
//
//         let mut icon_info = ICONINFO {
//             fIcon: 0,
//             xHotspot: 0,
//             yHotspot: 0,
//             hbmMask: ptr::null_mut(),
//             hbmColor: ptr::null_mut(),
//         };
//         GetIconInfo(shfi.hIcon, &mut icon_info);
//
//         let hdc = CreateCompatibleDC(ptr::null_mut());
//
//         let bmp_info_header = BITMAPINFOHEADER {
//             biSize: mem::size_of::<BITMAPINFOHEADER>() as DWORD,
//             biWidth: 32,
//             biHeight: -32, // Negative to indicate a top-down DIB
//             biPlanes: 1,
//             biBitCount: 32,
//             biCompression: BI_RGB as DWORD,
//             biSizeImage: 0,
//             biXPelsPerMeter: 0,
//             biYPelsPerMeter: 0,
//             biClrUsed: 0,
//             biClrImportant: 0,
//         };
//
//         let mut bitmap_info = BITMAPINFO {
//             bmiHeader: bmp_info_header,
//             bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }; 1],
//         };
//
//         let bitmap_memory = GlobalAlloc(GHND, (32 * 32 * 4) as usize);
//         let bitmap_bits = GlobalLock(bitmap_memory) as *mut u8;
//
//         GetDIBits(
//             hdc,
//             icon_info.hbmColor,
//             0,
//             32,
//             bitmap_bits as *mut _,
//             &mut bitmap_info,
//             0,
//         );
//
//         GlobalUnlock(bitmap_memory);
//         DeleteDC(hdc);
//         DeleteObject(icon_info.hbmColor as _);
//         DestroyIcon(shfi.hIcon);
//
//         let bitmap_slice = slice::from_raw_parts(bitmap_bits, (32 * 32 * 4) as usize).to_vec();
//
//         let mut rgba_slice = vec![0u8; bitmap_slice.len()];
//         for i in 0..(32 * 32) {
//             let b = bitmap_slice[i * 4 + 0];
//             let g = bitmap_slice[i * 4 + 1];
//             let r = bitmap_slice[i * 4 + 2];
//             let a = bitmap_slice[i * 4 + 3];
//             rgba_slice[i * 4 + 0] = r;
//             rgba_slice[i * 4 + 1] = g;
//             rgba_slice[i * 4 + 2] = b;
//             rgba_slice[i * 4 + 3] = a;
//         }
//
//         GlobalFree(bitmap_memory);
//
//         let handle = Handle::from_pixels(32, 32, rgba_slice);
//
//         return Ok(handle);
//     }
// }
