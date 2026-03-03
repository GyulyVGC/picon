#![doc = include_str!("../README.md")]

mod platform;

/// Indicates whether the current operating system is supported by this library.
///
/// Currently, the supported operating systems are macOS, Linux, and Windows.
pub const IS_OS_SUPPORTED: bool = cfg!(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows"
));

/// An Iced image or SVG handle of a process icon.
#[derive(Clone)]
pub enum IconHandle {
    Image(iced::widget::image::Handle),
    Svg(iced::widget::svg::Handle),
}

/// Returns the process icon given the path to an executable.
pub fn get_icon_by_path<S: Into<String>>(path: S) -> Option<IconHandle> {
    platform::get_icon_by_path(path.into())
}

#[cfg(test)]
mod tests {}
