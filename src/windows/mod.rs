/// Most of this was copied and adjusted from https://github.com/sindresorhus/windows-wallpaper.git.
/// Anything related to the `winreg` feature was ported from legacy behavior (about changing the Windows Registry).

#[cfg(feature = "winreg")]
mod legacy;
#[cfg(not(feature = "winreg"))]
mod modern;

#[cfg(not(feature = "winreg"))]
use std::mem::ManuallyDrop;
use std::path::PathBuf;

use crate::{error::Error, DesktopClient, Mode, Result};
#[cfg(not(feature = "winreg"))]
use windows::Win32::UI::Shell::IDesktopWallpaper;

#[derive(Debug)]
pub struct DesktopWallpaper {
    #[cfg(not(feature = "winreg"))]
    interface: ManuallyDrop<IDesktopWallpaper>,
}

impl DesktopClient for DesktopWallpaper {
    fn get_wallpaper(&self) -> Result<String> {
        #[cfg(feature = "winreg")]
        {
            legacy::get()
        }
        #[cfg(not(feature = "winreg"))]
        {
            modern::get(&self.interface, None)
        }
    }

    fn set_wallpaper(&mut self, path: &str, mode: Mode) -> Result<()> {
        let path = PathBuf::from(path)
            .canonicalize()
            .map_err(|_| Error::InvalidPath)?;

        #[cfg(feature = "winreg")]
        {
            legacy::set(&path, mode)
        }
        #[cfg(not(feature = "winreg"))]
        {
            modern::set(&self.interface, &path, mode)
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{DesktopClient, DesktopWallpaper};

    #[test]
    fn get_wallpaper() {
        let client = DesktopWallpaper::new().unwrap();
        let path = client.get_wallpaper().unwrap();
        let p = PathBuf::from(&path);
        println!("{p:?}");
        if !path.is_empty() {
            assert!(p.exists());
        }
    }
}
