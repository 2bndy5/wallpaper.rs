/// Most of this was copied and adjusted from https://github.com/sindresorhus/windows-wallpaper.git.
/// Anything related to the `winreg` feature was ported from legacy behavior (about changing the Windows Registry).
use std::{ffi::OsString, os::windows::prelude::OsStrExt, path::PathBuf};
#[cfg(feature = "winreg")]
use std::{
    ffi::{c_void, OsStr},
    io, iter, mem,
};
#[cfg(not(feature = "winreg"))]
use std::{mem::ManuallyDrop, os::windows::prelude::OsStringExt, path::Path};

use crate::{error::Error, DesktopClient, Mode, Result};
#[cfg(feature = "winreg")]
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
    SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};
use windows::{
    core,
    Win32::UI::Shell::{
        DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH,
        DWPOS_TILE,
    },
};
#[cfg(not(feature = "winreg"))]
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoFreeUnusedLibraries, CoInitialize, CoUninitialize,
            CLSCTX_LOCAL_SERVER,
        },
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
};
#[cfg(feature = "winreg")]
use windows_registry::CURRENT_USER;

impl From<Mode> for DESKTOP_WALLPAPER_POSITION {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Center => DWPOS_CENTER,
            Mode::Tile => DWPOS_TILE,
            Mode::Stretch => DWPOS_STRETCH,
            Mode::Fit => DWPOS_FIT,
            Mode::Crop => DWPOS_FILL,
            Mode::Span => DWPOS_SPAN,
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    pub monitor_index: OsString,
    pub wallpaper: OsString,
}

#[derive(Debug)]
pub struct DesktopWallpaper {
    #[cfg(not(feature = "winreg"))]
    interface: ManuallyDrop<IDesktopWallpaper>,
}

impl DesktopWallpaper {
    pub fn new() -> core::Result<Self> {
        #[cfg(not(feature = "winreg"))]
        let interface: IDesktopWallpaper = unsafe {
            CoInitialize(None)?;
            CoCreateInstance(&DesktopWallpaper, None, CLSCTX_LOCAL_SERVER)?
        };

        Ok(Self {
            #[cfg(not(feature = "winreg"))]
            interface: ManuallyDrop::new(interface),
        })
    }

    #[cfg(not(feature = "winreg"))]
    pub fn get_monitors(&self) -> core::Result<Vec<Monitor>> {
        let monitor_count = unsafe { self.interface.GetMonitorDevicePathCount()? };

        (0..monitor_count)
            .map(|index| -> core::Result<Monitor> {
                let monitor_index = unsafe {
                    OsString::from_wide(self.interface.GetMonitorDevicePathAt(index)?.as_wide())
                };
                let wallpaper = unsafe {
                    OsString::from_wide(
                        self.interface
                            .GetWallpaper(PCWSTR::from_raw(
                                monitor_index.encode_wide().collect::<Vec<u16>>().as_ptr(),
                            ))?
                            .as_wide(),
                    )
                };

                Ok(Monitor {
                    monitor_index,
                    wallpaper,
                })
            })
            .collect()
    }
}

#[cfg(all(test, not(feature = "winreg")))]
mod test_monitors {
    use super::DesktopWallpaper;

    #[test]
    fn list_monitors() {
        let desktop = DesktopWallpaper::new().unwrap();
        let monitors = desktop.get_monitors().unwrap();
        assert!(!monitors.is_empty());
        for monitor in monitors {
            println!("{monitor:#?}");
        }
    }
}

impl DesktopClient for DesktopWallpaper {
    fn get_wallpaper(&self) -> Result<String> {
        #[cfg(feature = "winreg")]
        {
            unsafe {
                let buffer: [u16; 260] = mem::zeroed();
                let successful = SystemParametersInfoW(
                    SPI_GETDESKWALLPAPER,
                    buffer.len() as u32,
                    Some(buffer.as_ptr() as *mut c_void),
                    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                ) == true;

                if successful {
                    let path = String::from_utf16(&buffer)
                        .map_err(|_| Error::InvalidPath)?
                        // removes trailing zeroes from buffer
                        .trim_end_matches('\x00')
                        .into();
                    Ok(path)
                } else {
                    Err(io::Error::last_os_error().into())
                }
            }
        }
        #[cfg(not(feature = "winreg"))]
        {
            let monitor_index = unsafe {
                OsString::from_wide(
                    self.interface
                        .GetMonitorDevicePathAt(0)
                        .map_err(|e| Error::IOError(e.into()))?
                        .as_wide(),
                )
            };
            let wallpaper: PWSTR = unsafe {
                self.interface
                    .GetWallpaper(PCWSTR(
                        monitor_index.encode_wide().collect::<Vec<u16>>().as_ptr(),
                    ))
                    .map_err(|e| Error::IOError(std::io::Error::from_raw_os_error(e.code().0)))?
            };

            let wallpaper_string = unsafe { OsString::from_wide(wallpaper.as_wide()) };

            let path = Path::new(&wallpaper_string);

            if path.exists() && path.is_file() {
                return Ok(path.to_string_lossy().to_string());
            }
            Err(Error::InvalidPath)
        }
    }

    fn set_wallpaper(&mut self, path: &str, mode: Mode) -> Result<()> {
        let path = PathBuf::from(path)
            .canonicalize()
            .map_err(|_| Error::InvalidPath)?;

        #[cfg(feature = "winreg")]
        {
            // set wallpaper mode
            let hkcu = CURRENT_USER
                .create("Control Panel\\Desktop")
                .map_err(|_e| Error::UnsupportedDesktop)?;
            hkcu.set_string(
                "TileWallpaper",
                &match mode {
                    Mode::Tile => "1",
                    _ => "0",
                }
                .to_string(),
            )
            .map_err(|e| Error::IOError(e.into()))?;

            // copied from https://searchfox.org/mozilla-central/rev/5e955a47c4af398e2a859b34056017764e7a2252/browser/components/shell/nsWindowsShellService.cpp#493
            hkcu.set_string(
                "WallpaperStyle",
                &match mode {
                    // does not work with integers
                    Mode::Center | Mode::Tile => "0",
                    Mode::Fit => "6",
                    Mode::Span => "22",
                    Mode::Stretch => "2",
                    Mode::Crop => "10",
                }
                .to_string(),
            )
            .map_err(|e| Error::IOError(e.into()))?;

            // set wallpaper
            unsafe {
                let path = OsStr::new(&path)
                    .encode_wide()
                    // append null byte
                    .chain(iter::once(0))
                    .collect::<Vec<u16>>();
                let successful = SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    Some(path.as_ptr() as *mut c_void),
                    SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
                ) == true;

                if successful {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error().into())
                }
            }
        }
        #[cfg(not(feature = "winreg"))]
        {
            let monitor_index = unsafe {
                OsString::from_wide(
                    self.interface
                        .GetMonitorDevicePathAt(0)
                        .map_err(|e| Error::IOError(e.into()))?
                        .as_wide(),
                )
            };
            unsafe {
                self.interface
                    .SetWallpaper(
                        &HSTRING::from(&monitor_index),
                        &HSTRING::from(path.as_os_str()),
                    )
                    .map_err(|e| Error::IOError(e.into()))?;

                self.interface
                    .SetPosition(mode.into())
                    .map_err(|e| Error::IOError(e.into()))?;
            }
            Ok(())
        }
    }
}

#[cfg(not(feature = "winreg"))]
impl Drop for DesktopWallpaper {
    fn drop(&mut self) {
        {
            unsafe {
                ManuallyDrop::drop(&mut self.interface);
                CoFreeUnusedLibraries();
                CoUninitialize();
            }
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
