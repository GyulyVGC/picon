use crate::Icon;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, HWND},
        Graphics::Gdi::{
            BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
            DeleteObject, GetDIBits, GetObjectW,
        },
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{
                DestroyIcon, GetDC, GetIconInfo, ReleaseDC,
                HICON, ICONINFO,
            },
        },
    },
};

pub(crate) fn get_icon_by_path(path: String) -> Option<Icon> {
    get_icon_bytes(&path).map(Icon::new)
}

pub fn get_icon_bytes(path: &Path) -> Option<Vec<u8>> {
    unsafe {
        // Convert path to null-terminated UTF-16
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut large_icon = HICON::default();

        let count = ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            Some(&mut large_icon),
            None,
            1,
        );

        if count == 0 || large_icon.is_invalid() {
            return None;
        }

        let mut icon_info = ICONINFO::default();
        if !GetIconInfo(large_icon, &mut icon_info).as_bool() {
            DestroyIcon(large_icon);
            return None;
        }

        let mut bmp = BITMAP::default();
        if GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bmp as *mut _ as *mut _,
        ) == 0
        {
            DestroyIcon(large_icon);
            return None;
        }

        let width = bmp.bmWidth;
        let height = bmp.bmHeight;

        let mut buffer = vec![0u8; (width * height * 4) as usize];

        let hdc = GetDC(HWND(0));

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        ReleaseDC(HWND(0), hdc);

        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);
        DestroyIcon(large_icon);

        if result == 0 {
            return None;
        }

        Some(buffer)
    }
}

