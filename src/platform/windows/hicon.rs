use crate::IconHandle;

use std::ffi::OsStr;
use std::mem::{self, MaybeUninit};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HBITMAP, HDC, HGDIOBJ, ReleaseDC,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, HICON, PrivateExtractIconsW,
};

// RAII wrappers for automatic resource cleanup

struct OwnedDc(HDC);

impl Drop for OwnedDc {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            let _ = unsafe { ReleaseDC(None, self.0) };
        }
    }
}

struct OwnedBitmap(HBITMAP);

impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            let _ = unsafe { DeleteObject(HGDIOBJ::from(self.0)) };
        }
    }
}

struct OwnedIcon(HICON);

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            let _ = unsafe { DestroyIcon(self.0) };
        }
    }
}

/// Extract an icon from an executable using the Windows Shell API.
pub(super) fn get_icon(exe_path: &str) -> Option<IconHandle> {
    let path = Path::new(exe_path);
    let hicon = unsafe { get_hicon(path) }?;
    let (w, h, rgba) = unsafe { hicon_to_rgba(hicon) }?;
    Some(IconHandle::Image(iced::widget::image::Handle::from_rgba(
        w, h, rgba,
    )))
}

unsafe fn get_hicon(file_path: &Path) -> Option<HICON> {
    let wide: Vec<u16> = OsStr::new(file_path).encode_wide().chain(Some(0)).collect();

    // PrivateExtractIconsW requires a fixed [u16; 260] (MAX_PATH) buffer.
    // Reserve the last element for the null terminator.
    let mut filename_buf = [0u16; 260];
    let copy_len = wide.len().min(filename_buf.len() - 1);
    filename_buf[..copy_len].copy_from_slice(&wide[..copy_len]);

    let mut hicons = [HICON::default()];
    let mut icon_id = 0u32;

    let count = unsafe {
        PrivateExtractIconsW(
            &filename_buf,
            0,
            64,
            64,
            Some(&mut hicons),
            Some(&mut icon_id),
            0,
        )
    };

    // PrivateExtractIconsW returns u32::MAX on failure, 0 if no icons found
    if count == 0 || count == u32::MAX || hicons[0].0.is_null() {
        return None;
    }

    Some(hicons[0])
}

unsafe fn hicon_to_rgba(icon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    let bitmap_size = i32::try_from(mem::size_of::<BITMAP>()).ok()?;
    let biheader_size = u32::try_from(mem::size_of::<BITMAPINFOHEADER>()).ok()?;

    // Take ownership of the icon immediately so it's destroyed on any early return
    let _icon_guard = OwnedIcon(icon);

    let mut info = MaybeUninit::uninit();
    unsafe { GetIconInfo(icon, info.as_mut_ptr()) }.ok()?;
    let info = unsafe { info.assume_init() };

    // GetIconInfo creates new bitmaps that the caller must delete (per MSDN)
    let _mask_guard = OwnedBitmap(info.hbmMask);
    let _color_guard = OwnedBitmap(info.hbmColor);

    let mut bitmap = MaybeUninit::<BITMAP>::uninit();
    let result = unsafe {
        GetObjectW(
            HGDIOBJ::from(info.hbmColor),
            bitmap_size,
            Some(bitmap.as_mut_ptr().cast()),
        )
    };
    if result != bitmap_size {
        return None;
    }
    let bitmap = unsafe { bitmap.assume_init() };

    let width = bitmap.bmWidth.unsigned_abs();
    let height = bitmap.bmHeight.unsigned_abs();
    let pixel_count = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?;

    let mut buf = vec![0u32; pixel_count];

    let dc = unsafe { GetDC(None) };
    if dc.0.is_null() {
        return None;
    }
    let _dc_guard = OwnedDc(dc);

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: biheader_size,
            biWidth: bitmap.bmWidth,
            biHeight: -bitmap.bmHeight, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        bmiColors: [Default::default()],
    };

    let scan_lines = unsafe {
        GetDIBits(
            dc,
            info.hbmColor,
            0,
            height,
            Some(buf.as_mut_ptr().cast()),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };
    if scan_lines == 0 {
        return None;
    }

    // BGRA -> RGBA
    let byte_len = buf.len().checked_mul(mem::size_of::<u32>())?;
    let rgba = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, byte_len) }
        .chunks_exact(4)
        .flat_map(|px| [px[2], px[1], px[0], px[3]])
        .collect();

    Some((width, height, rgba))
}
