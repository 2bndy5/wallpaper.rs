use std::path::PathBuf;

use crate::DesktopClient;
use crate::{error::Error, get_stdout, run, Mode, Result};

pub struct DesktopWallpaper;

impl DesktopWallpaper {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Drop for DesktopWallpaper {
    fn drop(&mut self) {}
}

impl DesktopClient for DesktopWallpaper {
    fn set_wallpaper(&mut self, img_path: &str, mode: Mode) -> Result<()> {
        let _ = mode; // Unable to change with AppleScript.
        let _ = PathBuf::from(img_path)
            .canonicalize()
            .map_err(|_| Error::InvalidPath)?;

        run(
            "osascript",
            &[
                "-e",
                format!(
                    r#"tell application "System Events" to tell every desktop to set picture to {}"#,
                    enquote::enquote('"', img_path),
                )
                .as_str(),
            ],
        )
    }

    fn get_wallpaper(&self) -> Result<String> {
        get_stdout(
            "osascript",
            &[
                "-e",
                r#"tell application "Finder" to get POSIX path of (get desktop picture as alias)"#,
            ],
        )
    }
}
