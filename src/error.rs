use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[cfg(unix)]
    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),

    #[cfg(windows)]
    #[error("Windows SDK error: {0}")]
    WindowsSdk(#[from] windows::core::Error),

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    #[error("Invalid INI: {0}")]
    #[cfg(target_os = "linux")]
    InvalidIni(#[from] ini::Error),

    #[error("Enquote error: {0}")]
    #[cfg(unix)]
    Enquote(#[from] enquote::Error),

    #[cfg(unix)]
    #[error("{command} exited with status code {code}")]
    CommandFailed { command: String, code: i32 },

    #[cfg(target_os = "linux")]
    #[error("Could not find config directory")]
    NoConfigDir,

    #[cfg(target_os = "linux")]
    #[error("No {0} image found")]
    NoImage(&'static str),

    #[cfg(target_os = "linux")]
    #[error("No desktops found")]
    XfceNoDesktops,

    #[error("Unsupported Desktop Environment")]
    UnsupportedDesktop,

    #[error("Invalid path")]
    InvalidPath,
}
