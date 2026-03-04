pub(crate) use target_os::get_icon;

/* ---------- windows ---------- */
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as target_os;

/* ----------- macos ----------- */
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as target_os;

/* -------- linux & bsd -------- */
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
mod linux;
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use linux as target_os;

/* ----------- other ----------- */
#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows"),
    not(target_os = "freebsd"),
    not(target_os = "openbsd"),
    not(target_os = "netbsd")
))]
mod unsupported;
#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows"),
    not(target_os = "freebsd"),
    not(target_os = "openbsd"),
    not(target_os = "netbsd")
))]
use unsupported as target_os;
