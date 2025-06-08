use crate::DesktopClient;
use crate::{error::Error, get_stdout, run, Mode, Result};

pub struct DesktopWallpaper;

impl DesktopWallpaper {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl DesktopClient for DesktopWallpaper {
    fn set_wallpaper(&mut self, path: &str, mode: Mode) -> Result<()> {
        let _ = mode; // Unable to change with AppleScript.

        run(
            "osascript",
            &[
                "-e",
                format!(
                    r#"tell application "System Events" to tell every desktop to set picture to {}"#,
                    enquote::enquote('"', path),
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

impl Drop for DesktopWallpaper {
    fn drop(&mut self) {}
}
