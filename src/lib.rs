#![doc = include_str!("../readme.md")]
mod error;
pub use error::Error;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;

#[cfg(all(unix, not(target_os = "macos")))]
pub use crate::linux::*;

// macos
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::*;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Center,
    Crop,
    Fit,
    Span,
    Stretch,
    Tile,
}

#[cfg(unix)]
fn get_stdout(command: &str, args: &[&str]) -> Result<String> {
    use std::process::Command;

    let output = Command::new(command).args(args).output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().into())
    } else {
        Err(Error::CommandFailed {
            command: command.to_string(),
            code: output.status.code().unwrap_or(-1),
        })
    }
}

#[cfg(unix)]
#[inline]
fn run(command: &str, args: &[&str]) -> Result<()> {
    get_stdout(command, args).map(|_| ())
}

pub trait DesktopClient {
    /// Sets the wallpaper from a file `path` with the given [`Mode`].
    fn set_wallpaper(&mut self, path: &str, mode: Option<Mode>) -> Result<()> {
        let _ = path;
        let _ = mode;
        Err(Error::UnsupportedDesktop)
    }
    /// Returns the current wallpaper.
    fn get_wallpaper(&self) -> Result<String> {
        Err(Error::UnsupportedDesktop)
    }
}
