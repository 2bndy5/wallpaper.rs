use std::{io, string::FromUtf8Error};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O Error: {0}")]
    IOError(#[from] io::Error),

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("Invalid INI: {0}")]
    #[cfg(all(unix, not(target_os = "macos")))]
    InvalidIni(#[from] ini::Error),

    #[error("Enquote error: {0}")]
    #[cfg(unix)]
    Enquote(#[from] enquote::Error),

    #[error("{command} exited with status code {code}")]
    CommandFailed { command: String, code: i32 },

    #[error("Could not find config directory")]
    NoConfigDir,

    #[error("No {0} image found")]
    NoImage(&'static str),

    #[cfg(all(unix, not(target_os = "macos")))]
    #[error("No desktops found")]
    XfceNoDesktops,

    #[error("Unsupported Desktop Environment")]
    UnsupportedDesktop,

    #[error("Invalid path")]
    InvalidPath,

    #[error("Cannot set  wallpaper mode3 on MacOS")]
    #[cfg(target_os = "macos")]
    MacOsUnsupportedMode,
}
