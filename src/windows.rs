#[cfg(feature = "winreg")]
pub mod winreg {
    use crate::{error, DesktopClient, Mode, Result};
    use std::ffi::OsStr;
    use std::io;
    use std::iter;
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
    use winapi::ctypes::c_void;
    use winapi::um::winuser::SystemParametersInfoW;
    use winapi::um::winuser::SPIF_SENDCHANGE;
    use winapi::um::winuser::SPIF_UPDATEINIFILE;
    use winapi::um::winuser::SPI_GETDESKWALLPAPER;
    use winapi::um::winuser::SPI_SETDESKWALLPAPER;
    use winreg::enums::*;
    use winreg::RegKey;

    pub struct Monitor(usize);

    impl From<usize> for Monitor {
        fn from(value: usize) -> Self {
            Self(value)
        }
    }

    #[derive(Debug, Default)]
    pub struct DesktopWallpaper;

    impl DesktopClient for DesktopWallpaper {
        fn set_wallpaper<M: Into<Monitor>>(
            &mut self,
            monitor: M,
            path: &str,
            mode: Mode,
        ) -> Result<()> {
            let _ = monitor;

            // set wallpaper mode
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (desktop, _) = hkcu.create_subkey(r"Control Panel\Desktop")?;

            desktop.set_value(
                "TileWallpaper",
                &match mode {
                    Mode::Tile => "1",
                    _ => "0",
                }
                .to_string(),
            )?;

            // copied from https://searchfox.org/mozilla-central/rev/5e955a47c4af398e2a859b34056017764e7a2252/browser/components/shell/nsWindowsShellService.cpp#493
            desktop.set_value(
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
            )?;

            // set wallpaper
            unsafe {
                let path = OsStr::new(path)
                    .encode_wide()
                    // append null byte
                    .chain(iter::once(0))
                    .collect::<Vec<u16>>();
                let successful = SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    path.as_ptr() as *mut c_void,
                    SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
                ) == 1;

                if successful {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error().into())
                }
            }
        }

        fn get_wallpaper<M: Into<Monitor>>(&self, monitor: M) -> Result<String> {
            let _ = monitor;
            unsafe {
                let buffer: [u16; 260] = mem::zeroed();
                let successful = SystemParametersInfoW(
                    SPI_GETDESKWALLPAPER,
                    buffer.len() as u32,
                    buffer.as_ptr() as *mut c_void,
                    0,
                ) == 1;

                if successful {
                    let path = String::from_utf16(&buffer)
                        .map_err(|_| error::Error::InvalidPath)?
                        // removes trailing zeroes from buffer
                        .trim_end_matches('\x00')
                        .into();
                    Ok(path)
                } else {
                    Err(io::Error::last_os_error().into())
                }
            }
        }
    }
}

/// copied and adjusted from https://github.com/sindresorhus/windows-wallpaper.git
pub mod win_rs {
    use std::{
        ffi::OsString,
        mem::ManuallyDrop,
        os::windows::prelude::{OsStrExt, OsStringExt},
        path::{Path, PathBuf},
    };

    use crate::{error::Error, DesktopClient, Mode, Result};
    use windows::{
        core::{self, HSTRING, PCWSTR, PWSTR},
        Win32::{
            System::Com::{
                CoCreateInstance, CoFreeUnusedLibraries, CoInitialize, CoUninitialize,
                CLSCTX_LOCAL_SERVER,
            },
            UI::Shell::{
                DesktopWallpaper, IDesktopWallpaper, DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER,
                DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH, DWPOS_TILE,
            },
        },
    };

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
        interface: ManuallyDrop<IDesktopWallpaper>,
    }
    impl Default for DesktopWallpaper {
        fn default() -> Self {
            Self::new().expect("Failed to create a desktop interface in Windows")
        }
    }

    impl DesktopWallpaper {
        pub fn new() -> core::Result<Self> {
            let interface: IDesktopWallpaper = unsafe {
                CoInitialize(None)?;
                CoCreateInstance(&DesktopWallpaper, None, CLSCTX_LOCAL_SERVER)?
            };

            Ok(Self {
                interface: ManuallyDrop::new(interface),
            })
        }

        /*
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
        */
    }

    impl DesktopClient for DesktopWallpaper {
        fn get_wallpaper(&self) -> Result<String> {
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

        fn set_wallpaper(&mut self, path: &str, position: Mode) -> Result<()> {
            let path = match PathBuf::from(path).canonicalize() {
                Ok(p) => p,
                // Could not get the canonical form of the path.
                Err(_) => return Err(Error::InvalidPath),
            };

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
                    .SetPosition(position.into())
                    .map_err(|e| Error::IOError(e.into()))?;
            }
            Ok(())
        }
    }

    impl Drop for DesktopWallpaper {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.interface);
                CoFreeUnusedLibraries();
                CoUninitialize();
            }
        }
    }
}
