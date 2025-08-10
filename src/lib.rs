#![doc = include_str!("../README.md")]

mod platform;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Indicates whether the current operating system is supported by this library.
///
/// Currently, the supported operating systems are macOS, Linux, and Windows.
pub const IS_OS_SUPPORTED: bool = cfg!(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows"
));

/// A process icon.
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Icon {
    pub bytes: Vec<u8>,
}

impl Icon {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Default for Icon {
    fn default() -> Self {
        Self { bytes: Vec::new() }
    }
}

/// Returns the process icon given the path to an executable.
pub fn get_icon_by_path<S: Into<String>>(path: S) -> Icon {
    platform::get_icon_by_path(path.into()).unwrap_or_default()
}

#[cfg(test)]
mod tests {}
