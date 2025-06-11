//! Most of this was copied and adjusted from https://github.com/sindresorhus/windows-wallpaper.git.
//! Basically, this is everything related to using the Windows SDK (instead of Windows Registry).

use super::DesktopWallpaper;
use crate::{Error, Mode, Result};
use std::{
    ffi::OsString,
    mem::ManuallyDrop,
    os::windows::{ffi::OsStrExt, prelude::OsStringExt},
    path::Path,
};
use windows::{
    core::{self, HSTRING, PCWSTR, PWSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoFreeUnusedLibraries, CoInitialize, CoUninitialize,
            CLSCTX_LOCAL_SERVER,
        },
        UI::Shell::{
            IDesktopWallpaper, DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT,
            DWPOS_SPAN, DWPOS_STRETCH, DWPOS_TILE,
        },
    },
};

#[derive(Debug)]
pub struct Monitor {
    pub monitor_index: OsString,
    pub wallpaper: OsString,
}

impl DesktopWallpaper {
    pub fn new() -> core::Result<Self> {
        let interface: IDesktopWallpaper = unsafe {
            let init = CoInitialize(None);
            if init.is_err() {
                return Err(core::Error::from_hresult(init));
            }
            CoCreateInstance(
                &windows::Win32::UI::Shell::DesktopWallpaper,
                None,
                CLSCTX_LOCAL_SERVER,
            )?
        };

        Ok(Self {
            interface: ManuallyDrop::new(interface),
        })
    }

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

    pub fn set_with_monitors(
        &self,
        path: &Path,
        mode: Mode,
        monitors: Option<&[OsString]>,
    ) -> Result<()> {
        let monitor_count = unsafe { self.interface.GetMonitorDevicePathCount()? };
        for i in 0..monitor_count {
            let monitor_index =
                unsafe { OsString::from_wide(self.interface.GetMonitorDevicePathAt(i)?.as_wide()) };

            // if no monitors specified at all or monitor is specified
            if monitors.is_none_or(|m| m.contains(&monitor_index)) {
                // set wallpaper for every monitor
                unsafe {
                    self.interface.SetWallpaper(
                        &HSTRING::from(&monitor_index),
                        &HSTRING::from(path.as_os_str()),
                    )?;
                }
            }
        }

        // set wallpaper mode
        unsafe {
            self.interface.SetPosition(mode.into())?;
        }
        Ok(())
    }
}

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

pub fn get(interface: &ManuallyDrop<IDesktopWallpaper>, monitor: Option<u32>) -> Result<String> {
    let monitor = match monitor {
        None => 0,
        Some(m) => {
            let max_monitor = unsafe { interface.GetMonitorDevicePathCount()? };
            m.min(max_monitor - 1)
        }
    };
    let monitor_index =
        unsafe { OsString::from_wide(interface.GetMonitorDevicePathAt(monitor)?.as_wide()) };
    let wallpaper: PWSTR = unsafe {
        interface.GetWallpaper(PCWSTR(
            monitor_index.encode_wide().collect::<Vec<u16>>().as_ptr(),
        ))?
    };

    let wallpaper_string = unsafe { OsString::from_wide(wallpaper.as_wide()) };

    let path = Path::new(&wallpaper_string);

    if path.exists() && path.is_file() {
        return Ok(path.to_string_lossy().to_string());
    }
    Err(Error::InvalidPath)
}

#[cfg(test)]
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
