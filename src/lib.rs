#![doc = include_str!("../README.md")]

use iced::widget::image;

mod platform;

/// Indicates whether the current operating system is supported by this library.
///
/// Currently, the supported operating systems are macOS, Linux, and Windows.
pub const IS_OS_SUPPORTED: bool = cfg!(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows"
));

/// Returns the process icon given the path to an executable.
pub fn get_icon_by_path<S: Into<String>>(path: S) -> Option<image::Handle> {
    platform::get_icon_by_path(path.into())
}

#[cfg(test)]
mod tests {}
