use crate::Icon;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, path::Path};

pub(crate) fn get_icon_by_path(path: String) -> Option<Icon> {
    get_icon_bytes(&path).map(Icon::new)
}

fn get_icon_bytes(path: &str) -> Option<Vec<u8>> {
    unsafe {
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut large_icon = HICON::default();

        let extracted = ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            Some(&mut large_icon),
            None,
            1,
        );

        if extracted == 0 {
            return None;
        }

        let mut icon_info = ICONINFO::default();
        GetIconInfo(large_icon, &mut icon_info);

        let mut bmp = BITMAP::default();
        GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bmp as *mut _ as *mut _,
        );

        let mut buffer = vec![0u8; (bmp.bmWidth * bmp.bmHeight * 4) as usize];

        let hdc = GetDC(HWND(0));
        GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            bmp.bmHeight as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: bmp.bmWidth,
                    biHeight: -bmp.bmHeight,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB as u32,
                    ..Default::default()
                },
                ..Default::default()
            },
            DIB_RGB_COLORS,
        );

        ReleaseDC(HWND(0), hdc);

        Some(buffer)
    }
}

